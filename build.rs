fn main() {
    let output = std::process::Command::new("git")
        .args(["describe", "--exact-match", "--tags", "HEAD"])
        .output();
    let is_tagged = output.map(|o| o.status.success()).unwrap_or(false);
    println!("cargo:rustc-env=APP_IS_TAGGED={is_tagged}");

    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=APP_GIT_SHA={}", sha.trim());
}
