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
        let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
        let obj_path = format!("{out_dir}/memset_s_shim.o");

        // Compile directly to a .o file and link it as a positional linker arg.
        // This sidesteps the static-archive ordering problem: a bare -l flag only
        // pulls symbols that are already undefined at that point in the link,
        // but libsodium's reference to memset_explicit comes later. A positional
        // object file is always included unconditionally.
        let compiler = cc::Build::new().get_compiler();
        let status = compiler
            .to_command()
            .args(["-DWIN32", "-D__WIN32__", "-c"])
            .arg("-o")
            .arg(&obj_path)
            .arg("memset_s_shim.c")
            .status()
            .expect("failed to invoke C compiler for memset_s_shim.c");
        assert!(status.success(), "failed to compile memset_s_shim.c");

        println!("cargo:rerun-if-changed=memset_s_shim.c");
        println!("cargo:rustc-link-arg={obj_path}");
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
