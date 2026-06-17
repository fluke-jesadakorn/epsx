// Wave 28 T1 — run Tailwind v4 PostCSS build before cargo build.
//
// Cargo invokes `build.rs` with CWD set to the package root
// (apps/frontend/), so `Command::new("bun").args(["run", "build:css"])`
// already runs from the right place — the package's own package.json
// is discovered by `bun run`.
//
// We only run the CSS build during release builds (production binary)
// to keep `cargo check` + `cargo test` fast. Dev builds (cargo run /
// cargo build without --release) skip the CSS build — devs run
// `bun run build:css` explicitly when iterating on styles.

use std::process::Command;

fn main() {
    // Re-run if the CSS source / config / package.json / output changes.
    println!("cargo:rerun-if-changed=src/styles/index.css");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=postcss.config.cjs");
    println!("cargo:rerun-if-changed=public/dist/tailwind.css");

    // Only run during cargo build (release). `cargo check` + `cargo test`
    // skip this to keep the inner loop fast.
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile != "release" {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());

    // Try `bun` first (project's packageManager), fall back to `npm`.
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