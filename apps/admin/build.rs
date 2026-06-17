// Wave 28 T1 — run Tailwind v4 PostCSS build before cargo build.
//
// See `apps/frontend/build.rs` for the design rationale.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/styles/index.css");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=postcss.config.cjs");
    println!("cargo:rerun-if-changed=public/dist/tailwind.css");

    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let mut cmd = Command::new("bun");
    cmd.args(["run", "build:css"]).current_dir(&manifest_dir);
    let status = cmd.status();

    let success = match status {
        Ok(s) => s.success(),
        Err(_) => false,
    };

    if !success {
        eprintln!("cargo:warning=bun run build:css failed or bun missing — falling back to npm");
        let fallback = Command::new("npm")
            .args(["run", "build:css"])
            .current_dir(&manifest_dir)
            .status();
        if let Ok(s) = fallback {
            if !s.success() {
                eprintln!(
                    "cargo:warning=npm run build:css also failed — public/dist/tailwind.css \
                     may be stale (build.rs continuing)"
                );
            }
        }
    }
}