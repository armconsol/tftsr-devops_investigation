fn main() {
    let version = get_version_from_git();

    println!("cargo:rustc-env=APP_VERSION={}", version);
    println!("cargo:rerun-if-changed=.git/refs/heads/master");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    tauri_build::build()
}

fn get_version_from_git() -> String {
    if let Ok(output) = std::process::Command::new("git")
        .arg("describe")
        .arg("--tags")
        .arg("--abbrev=0")
        .output()
    {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout)
                .trim()
                .trim_start_matches('v')
                .to_string();
            if !version.is_empty() {
                return version;
            }
        }
    }

    "0.2.50".to_string()
}
