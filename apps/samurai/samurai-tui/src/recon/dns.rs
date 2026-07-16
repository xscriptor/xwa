use crate::recon::DnsRecord;
use hickory_resolver::TokioAsyncResolver;

pub async fn enumerate_dns(
    target: &str,
    resolver: &TokioAsyncResolver,
) -> Vec<DnsRecord> {
    let mut records = Vec::new();

    let types = [
        ("A", hickory_resolver::proto::rr::RecordType::A),
        ("AAAA", hickory_resolver::proto::rr::RecordType::AAAA),
        ("MX", hickory_resolver::proto::rr::RecordType::MX),
        ("NS", hickory_resolver::proto::rr::RecordType::NS),
        ("TXT", hickory_resolver::proto::rr::RecordType::TXT),
        ("SOA", hickory_resolver::proto::rr::RecordType::SOA),
        ("CNAME", hickory_resolver::proto::rr::RecordType::CNAME),
    ];

    for (name, rtype) in &types {
        if let Ok(response) = resolver.lookup(target, *rtype).await {
            for record in response.record_iter() {
                let value = match *rtype {
                    hickory_resolver::proto::rr::RecordType::A => {
                        record.data().and_then(|d| d.as_a()).map(|a| a.to_string()).unwrap_or_default()
                    }
                    hickory_resolver::proto::rr::RecordType::AAAA => {
                        record.data().and_then(|d| d.as_aaaa()).map(|a| a.to_string()).unwrap_or_default()
                    }
                    _ => record.data().map(|d| d.to_string()).unwrap_or_default(),
                };

                if !value.is_empty() {
                    records.push(DnsRecord {
                        record_type: name.to_string(),
                        name: record.name().to_string(),
                        value,
                    });
                }
            }
        }
    }

    records
}
