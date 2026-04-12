pub fn run() {
    println!("apm {} ({})", env!("CARGO_PKG_VERSION"), env!("APM_GIT_DESCRIBE"));
}
