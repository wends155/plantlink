use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../ui/src");
    println!("cargo:rerun-if-changed=../ui/package.json");
    println!("cargo:rerun-if-changed=../ui/svelte.config.js");
    println!("cargo:rerun-if-changed=../ui/vite.config.js");

    // Check if we need to build the UI
    // We build if:
    // 1. We are in release mode
    // 2. The dist folder is missing
    let dist_path = Path::new("../ui/dist");
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let should_build = profile == "release" || !dist_path.exists();

    if should_build {
        println!(
            "cargo:warning=Building UI assets for {} profile...",
            profile
        );

        let shell = if cfg!(target_os = "windows") {
            "cmd"
        } else {
            "sh"
        };
        let arg1 = if cfg!(target_os = "windows") {
            "/C"
        } else {
            "-c"
        };

        // npm install
        let install_cmd = "cd ../ui && npm install".to_string();
        let status = Command::new(shell)
            .args([arg1, &install_cmd])
            .status()
            .expect("Failed to run npm install");

        if !status.success() {
            panic!("Frontend npm install failed");
        }

        // npm run build
        let build_cmd = "cd ../ui && npm run build".to_string();
        let status = Command::new(shell)
            .args([arg1, &build_cmd])
            .status()
            .expect("Failed to run npm run build");

        if !status.success() {
            panic!("Frontend npm run build failed");
        }
    } else {
        println!(
            "cargo:warning=Skipping UI build (Debug mode & dist exists). Run 'npm run build' manually if needed."
        );
    }
}
