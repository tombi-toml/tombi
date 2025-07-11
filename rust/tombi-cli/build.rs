use std::process::Command;

fn main() {
    // Try to get version from git tag
    let git_version = Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                if let Some(tag) = String::from_utf8(output.stdout).ok() {
                    if tag.starts_with("v") {
                        return Some(tag.trim().to_string());
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=__TOMBI_VERSION={}", git_version);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
