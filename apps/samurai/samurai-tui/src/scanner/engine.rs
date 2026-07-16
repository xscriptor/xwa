use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;
use std::time::Duration;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ScanEvent {
    Log(String),
    Error(String),
    Done { open_ports: Vec<PortInfo> },
}

#[derive(Debug, Clone)]
pub struct PortInfo {
    pub port: String,
    pub service: String,
    pub raw_line: String,
}

pub async fn run_nmap_streaming(
    target: &str,
    profile: &str,
    nmap_proxy: Option<&str>,
    sender: UnboundedSender<ScanEvent>,
) {
    let mut args: Vec<&str> = vec!["-T4", "--min-rate", "1000"];

    match profile {
        "deep" => {
            args.extend_from_slice(&[
                "-sV",
                "-sC",
                "-p-",
                "--script",
                "vuln",
                "--host-timeout",
                "600s",
            ]);
        }
        "balanced" => {
            args.extend_from_slice(&[
                "-sV",
                "-sC",
                "-p",
                "1-10000",
                "--host-timeout",
                "300s",
            ]);
        }
        "udp" => {
            args.extend_from_slice(&["-sU", "-sV", "--top-ports", "1000", "--host-timeout", "300s"]);
        }
        _ => {
            args.extend_from_slice(&["-sV", "-sC", "--top-ports", "1000", "--host-timeout", "180s"]);
        }
    }

    // Optional nmap proxy (SAMURAI_NMAP_PROXY)
    if let Some(proxy) = nmap_proxy {
        args.extend_from_slice(&["--proxies", proxy]);
    }

    let mut child = match Command::new("nmap")
        .arg(target)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = sender.send(ScanEvent::Error(format!("Failed to spawn nmap: {}", e)));
            return;
        }
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => {
            let _ = sender.send(ScanEvent::Error("nmap stdout not available".into()));
            let _ = child.kill().await;
            return;
        }
    };
    let stderr = match child.stderr.take() {
        Some(s) => s,
        None => {
            let _ = sender.send(ScanEvent::Error("nmap stderr not available".into()));
            let _ = child.kill().await;
            return;
        }
    };

    let mut reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();

    let sender_stderr = sender.clone();
    let stderr_handle = tokio::spawn(async move {
        while let Ok(Some(line)) = err_reader.next_line().await {
            let _ = sender_stderr.send(ScanEvent::Log(format!("[stderr] {}", line)));
        }
    });

    let mut open_ports: Vec<PortInfo> = Vec::new();
    let sender_clone = sender.clone();

    let timeout_result = tokio::time::timeout(
        Duration::from_secs(600),
        async {
            while let Ok(Some(line)) = reader.next_line().await {
                let _ = sender_clone.send(ScanEvent::Log(line.clone()));

                if line.contains("/tcp") && line.contains("open") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let port = parts[0].to_string();
                        let service = parts[2..].join(" ");
                        open_ports.push(PortInfo {
                            port: port.clone(),
                            service: service.clone(),
                            raw_line: line.clone(),
                        });
                    }
                }
            }
            open_ports
        },
    )
    .await;

    // Abort stderr reader — we don't care about the result
    stderr_handle.abort();

    let _ = child.kill().await;

    match timeout_result {
        Ok(ports) => {
            let _ = sender.send(ScanEvent::Done { open_ports: ports });
        }
        Err(_) => {
            let _ = sender.send(ScanEvent::Error("Scan timed out after 600s".into()));
        }
    }
}
