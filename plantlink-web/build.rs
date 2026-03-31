use std::env;
use std::path::Path;
use std::process::Command;

// Justification: Build scripts have no runtime recovery path. If npm install fails, the build must abort.
#[allow(clippy::expect_used)]
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
    // ast-grep-ignore: scattered-env-var
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let should_build = profile == "release" || !dist_path.exists();

    if should_build {
        println!("cargo:warning=Building UI assets for {profile} profile...");
        // Set working directory for UI build
        let ui_dir = Path::new("../ui");

        let npm_cmd = if cfg!(target_os = "windows") {
            "npm.cmd"
        } else {
            "npm"
        };

        // npm install
        let status = Command::new(npm_cmd)
            .arg("install")
            .current_dir(ui_dir)
            .status()
            .expect("Failed to run npm install");

        assert!(status.success(), "Frontend npm install failed");

        // npm run build
        let status = Command::new(npm_cmd)
            .args(["run", "build"])
            .current_dir(ui_dir)
            .status()
            .expect("Failed to run npm run build");

        assert!(status.success(), "Frontend npm run build failed");
    } else {
        println!(
            "cargo:warning=Skipping UI build (Debug mode & dist exists). Run 'npm run build' manually if needed."
        );
    }
}
