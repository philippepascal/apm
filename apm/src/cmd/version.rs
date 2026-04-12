pub fn run() {
    let version = env!("CARGO_PKG_VERSION");
    let build = if cfg!(debug_assertions) { "dev" } else { "release" };
    println!("apm {} ({})", version, build);
}
