use std::path::Path;

fn main() {
    let ui_dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../apm-ui/dist");
    if !ui_dist.exists() {
        std::fs::create_dir_all(&ui_dist).expect("failed to create apm-ui/dist stub");
        std::fs::write(
            ui_dist.join("index.html"),
            "<html><body>UI not built. Run <code>npm run build</code> in apm-ui/.</body></html>\n",
        )
        .expect("failed to write stub index.html");
    }
    println!("cargo::rerun-if-changed=../apm-ui/dist");
}
