use std::path::Path;

pub fn run(root: &Path, hook_name: &str) {
    match hook_name {
        "pre-push" => pre_push(root),
        other => eprintln!("apm _hook: unknown hook {:?}", other),
    }
}

fn pre_push(_root: &Path) {
    // Auto-transition on branch push has been removed.
    // State advances via explicit apm commands only.
}
