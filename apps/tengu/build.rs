fn main() {
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/angular.json");
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=build.rs");
}
