fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let target = std::env::var("TARGET").unwrap_or_default();
    if !target.contains("linux") {
        panic!("System Logs Diagnostic only supports Linux (systemd required).");
    }
}
