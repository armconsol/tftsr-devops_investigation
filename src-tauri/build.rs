fn main() {
    let version = get_version_from_git();

    println!("cargo:rustc-env=APP_VERSION={version}");
    println!("cargo:rerun-if-changed=.git/refs/heads/master");
    println!("cargo:rerun-if-changed=.git/refs/tags");

    // Compile memset_explicit shim for Windows MinGW
    // libsodium-sys-stable uses memset_explicit which isn't available in MinGW
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    
    if target_os == "windows" && target_env == "gnu" {
        cc::Build::new()
            .file("memset_s_shim.c")
            .define("WIN32", None)
            .define("__WIN32__", None)
            .compile("memset_shim");
        println!("cargo:rerun-if-changed=memset_s_shim.c");
        // Link the shim library - must be before libsodium in link order
        // The shim provides memset_explicit which libsodium expects
        println!("cargo:rustc-link-lib=static=memset_shim");
    }

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
