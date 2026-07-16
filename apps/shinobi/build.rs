use std::process::Command;

fn main() {
    let frontend_dir = std::path::Path::new("frontend");
    if !frontend_dir.join("package.json").exists() {
        println!("cargo:warning=Angular frontend not found, using static/ fallback");
        return;
    }

    println!("cargo:warning=Building Angular frontend...");

    let npm_install = Command::new("npm")
        .args(["install", "--legacy-peer-deps"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm install");

    if !npm_install.success() {
        println!("cargo:warning=npm install failed, using static/ fallback");
        return;
    }

    let ng_build = Command::new("npx")
        .args(["ng", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run ng build");

    if ng_build.success() {
        println!("cargo:warning=Angular build completed successfully");
    } else {
        println!("cargo:warning=ng build failed, using static/ fallback");
    }

    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/angular.json");
}
