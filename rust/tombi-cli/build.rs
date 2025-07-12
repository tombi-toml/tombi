use std::process::Command;

fn main() {
    let re = regex::Regex::new(r"^v\d+\.\d+\.\d+$").unwrap();

    // Try to get version from git tag
    let git_version = Command::new("git")
        .args(["tag", "--points-at", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                if let Some(tags) = String::from_utf8(output.stdout).ok() {
                    for tag in tags.split('\n') {
                        if re.is_match(tag.trim()) {
                            return Some(tag.trim().to_string());
                        }
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

    println!("cargo:rustc-env=__TOMBI_VERSION={}", git_version);
    println!("cargo:rerun-if-changed=.git/HEAD");
}
