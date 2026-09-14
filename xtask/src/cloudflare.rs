//! Cloudflare local simulation (wrangler workerd/miniflare) — replaces Colima K8s for local.
//! `cargo xtask cloudflare dev --local` wraps `bunx wrangler dev --local --persist-to=.wrangler/state`
//! `cargo xtask cloudflare build` wraps `cargo xtask browser-runtime build` + `cargo build --release`

use std::{path::PathBuf, process::Command};

fn repo_root() -> Result<PathBuf, String> {
    let mut current = std::env::current_dir().map_err(|e| format!("could not get cwd: {e}"))?;
    loop {
        if current.join("Cargo.toml").is_file() && current.join("xtask").is_dir() {
            return Ok(current);
        }
        if !current.pop() {
            return Err("could not locate repository root".into());
        }
    }
}

fn run_status(cmd: &mut Command, label: &str) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("could not start {label}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with {status}"))
    }
}

pub fn cloudflare(flags: &[String]) -> Result<(), String> {
    if flags.first().map(String::as_str) == Some("--help")
        || flags.first().map(String::as_str) == Some("-h")
        || flags.is_empty()
    {
        println!("cargo xtask cloudflare dev --local          run workerd/miniflare local (bunx wrangler dev --local --persist-to=.wrangler/state, Hybrid PG+D1) — single frontend");
        println!("cargo xtask cloudflare dev --local --all    run all 4 workerd (frontend:8787 backend:8788 admin:8789 pay:8790) — primary smoke gate");
        println!("cargo xtask cloudflare dev --local --all --hmr  hybrid: dx HMR :3000/:3001 + workerd backend:8788/pay:8790 + tailwind --watch + wasm --watch — prod-like + hot reload + Cloudflare storage");
        println!("cargo xtask cloudflare build          browser-runtime + cargo build --release (Cloudflare-ready)");
        return Ok(());
    }
    match flags.first().map(String::as_str) {
        Some("dev") => cloudflare_dev(&flags[1..]),
        Some("build") => cloudflare_build(&flags[1..]),
        Some(other) => Err(format!("cloudflare accepts dev|build, got {other}")),
        None => Err("cloudflare requires dev or build".into()),
    }
}

fn cloudflare_dev(flags: &[String]) -> Result<(), String> {
    let local = flags.iter().any(|f| f == "--local");
    if !local {
        return Err("cloudflare dev requires --local (wraps `bunx wrangler dev --local --persist-to=.wrangler/state`)".into());
    }
    let all = flags.iter().any(|f| f == "--all");
    let hmr = flags.iter().any(|f| f == "--hmr");
    // Validate flag combos
    if hmr && !all {
        return Err(
            "cloudflare dev --hmr requires --all (hybrid needs both dx and workerd)".into(),
        );
    }
    if hmr {
        return cloudflare_dev_hmr(flags);
    }
    let root = repo_root()?;
    let persist = root.join(".wrangler/state");
    std::fs::create_dir_all(&persist)
        .map_err(|e| format!("could not create {}: {e}", persist.display()))?;
    let services = ["frontend", "admin", "pay", "backend"];
    println!(
        "cloudflare dev --local{}: workerd/miniflare with --persist-to={}",
        if all { " --all" } else { "" },
        persist.display()
    );
    println!("  bindings: HYPERDRIVE_CORE→host.docker.internal:5432 (pool≤5), R2/KV/D1 file-persisted in .wrangler/state");
    println!("  run individually:");
    for svc in services {
        println!("    bunx wrangler dev --local --persist-to=.wrangler/state --config apps/{}/wrangler.jsonc", svc);
    }
    let has_wrangler = Command::new("bunx")
        .args(["wrangler", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !has_wrangler {
        println!("wrangler not found — install with `bun add -d wrangler` or `bun install` at repo root, then re-run.");
        return Ok(());
    }
    if all {
        println!("wrangler detected — launching all 4 services (frontend:8787 backend:8788 admin:8789 pay:8790) — ctrl-c to stop");
        let mut children = Vec::new();
        for svc in services {
            let config = format!("apps/{}/wrangler.jsonc", svc);
            let mut cmd = Command::new("bunx");
            cmd.args([
                "wrangler",
                "dev",
                "--local",
                "--persist-to=.wrangler/state",
                "--config",
                &config,
                "--inspector-port=0",
            ]);
            cmd.current_dir(&root);
            match cmd.spawn() {
                Ok(child) => children.push((svc, child)),
                Err(e) => return Err(format!("could not start {svc}: {e}")),
            }
            println!("  spawned {svc} -> {config} --inspector-port=0");
        }
        // Wait for any child to exit
        loop {
            let mut exited: Option<(usize, String, std::process::ExitStatus)> = None;
            for (idx, (svc, child)) in children.iter_mut().enumerate() {
                if let Some(status) = child
                    .try_wait()
                    .map_err(|e| format!("could not inspect {svc}: {e}"))?
                {
                    exited = Some((idx, svc.to_string(), status));
                    break;
                }
            }
            if let Some((idx, svc, status)) = exited {
                for (j, (_, c)) in children.iter_mut().enumerate() {
                    if j != idx {
                        let _ = c.kill();
                    }
                }
                return Err(format!("{svc} exited with {status}"));
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    } else {
        println!("wrangler detected — launching frontend dev (ctrl-c to stop, or run individual wrangler commands above)");
        println!("  tip: use --all to launch all 4 services");
        run_status(
            Command::new("bunx")
                .args([
                    "wrangler",
                    "dev",
                    "--local",
                    "--persist-to=.wrangler/state",
                    "--inspector-port=0",
                    "--config",
                    "apps/frontend/wrangler.jsonc",
                ])
                .current_dir(&root),
            "wrangler dev --local (frontend)",
        )?;
    }
    Ok(())
}

fn cloudflare_dev_hmr(_flags: &[String]) -> Result<(), String> {
    let root = repo_root()?;
    let persist = root.join(".wrangler/state");
    std::fs::create_dir_all(&persist)
        .map_err(|e| format!("could not create {}: {e}", persist.display()))?;
    println!("cloudflare dev --local --all --hmr: hybrid prod-like + hot reload");
    println!("  frontend: dx serve --hot-reload :3000 (remote API https://dev-api.epsx.io) — watch_path 9 crates");
    println!("  admin:    dx serve --hot-reload :3001 (remote API https://dev-api.epsx.io)");
    println!("  backend:  workerd :8788 (Hyperdrive+3xD1/KV/R2, --inspector-port=0)");
    println!("  pay:      workerd :8790 (PAYMENTS_D1, --inspector-port=0)");
    println!("  tailwind: bunx tailwindcss --watch ×2 (frontend/admin)");
    println!(
        "  wasm:     cargo watch -w browser-runtime -w service-worker -x browser-runtime build"
    );
    println!("  tunnel:   dev.epsx.io→3000 dev-admin→3001 dev-api→8788 dev-pay→8790 (cloudflared-config.dev.yml)");
    println!("  api:      Dioxus.toml proxy + API_URL → https://dev-api.epsx.io (remote)");

    // Check toolchain
    for (bin, args) in [
        ("bunx", vec!["wrangler", "--version"]),
        ("dx", vec!["--version"]),
        ("cargo", vec!["--version"]),
    ] {
        let ok = Command::new(bin)
            .args(&args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ok {
            println!("{bin} not found — install missing toolchain then re-run");
            return Ok(());
        }
    }

    // Build browser-runtime first (wasm-bindgen)
    println!("  building browser-runtime (wasm32-unknown-unknown) ...");
    if let Err(e) = crate::node_free::browser_runtime(&["build".to_string()]) {
        return Err(format!("browser-runtime build failed: {e}"));
    }

    use std::process::{Child, Stdio};
    let mut children: Vec<(&'static str, Child)> = Vec::with_capacity(8);

    // Helper to spawn wrangler with --inspector-port=0
    let spawn_wrangler =
        |svc: &'static str, config: &str, root: &PathBuf| -> Result<Child, String> {
            let mut cmd = Command::new("bunx");
            cmd.args([
                "wrangler",
                "dev",
                "--local",
                "--persist-to=.wrangler/state",
                "--inspector-port=0",
                "--config",
                config,
            ]);
            cmd.current_dir(root);
            cmd.spawn()
                .map_err(|e| format!("could not start wrangler {svc}: {e}"))
        };

    // Backend 8788
    children.push((
        "workerd-backend",
        spawn_wrangler("backend", "apps/backend/wrangler.jsonc", &root)?,
    ));
    println!("  spawned workerd-backend -> apps/backend/wrangler.jsonc :8788 --inspector-port=0");
    // Pay 8790
    children.push((
        "workerd-pay",
        spawn_wrangler("pay", "apps/pay/wrangler.jsonc", &root)?,
    ));
    println!("  spawned workerd-pay -> apps/pay/wrangler.jsonc :8790 --inspector-port=0");

    // Dx frontend :3000 / admin :3001 with remote API env — dx 0.7.9 requires --hot-reload true (value) and --port
    let spawn_dx = |label: &'static str, app_dir: PathBuf| -> Result<Child, String> {
        let port = if label == "dx-frontend" {
            "3000"
        } else {
            "3001"
        };
        let mut cmd = Command::new("dx");
        cmd.args([
            "serve",
            "--hot-reload",
            "true",
            "--port",
            port,
            "--addr",
            "127.0.0.1",
        ])
        .current_dir(&app_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
        // Local workerd API for CLI-only without Docker/Tunnel — use http://localhost:8788, remote https://dev-api.epsx.io requires cloudflared tunnel + valid token
        cmd.env("ENV", "local");
        cmd.env("EPSX_ENV", "local");
        cmd.env("API_URL", "http://localhost:8788");
        cmd.env("BACKEND_URL", "http://localhost:8788");
        cmd.env("OIDC_ISSUER", "http://localhost:8788");
        cmd.env(
            "FRONTEND_URL",
            if label == "dx-frontend" {
                "http://localhost:3000"
            } else {
                "http://localhost:3001"
            },
        );
        cmd.env("HOST", "127.0.0.1");
        cmd.env("PORT", port);
        cmd.env("PAY_SERVICE_URL", "http://localhost:8790");
        cmd.spawn()
            .map_err(|e| format!("could not start {label}: {e}"))
    };

    let front_dir = root.join("apps/frontend");
    let admin_dir = root.join("apps/admin");
    children.push(("dx-frontend", spawn_dx("dx-frontend", front_dir)?));
    println!(
        "  spawned dx-frontend -> apps/frontend :3000 --hot-reload (proxy https://dev-api.epsx.io)"
    );
    children.push(("dx-admin", spawn_dx("dx-admin", admin_dir)?));
    println!("  spawned dx-admin -> apps/admin :3001 --hot-reload (proxy https://dev-api.epsx.io)");

    // Tailwind --watch ×2
    let spawn_tailwind = |app: &'static str, dir: PathBuf| -> Result<Child, String> {
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg("bunx tailwindcss -i ./src/styles/index.css -o ./public/dist/tailwind.css --watch")
            .current_dir(dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        cmd.spawn()
            .map_err(|e| format!("could not start tailwind-{app}: {e}"))
    };
    children.push((
        "tailwind-frontend",
        spawn_tailwind("frontend", root.join("apps/frontend"))?,
    ));
    println!("  spawned tailwind-frontend --watch");
    children.push((
        "tailwind-admin",
        spawn_tailwind("admin", root.join("apps/admin"))?,
    ));
    println!("  spawned tailwind-admin --watch");

    // Wasm watch
    let mut wasm_cmd = Command::new("cargo");
    wasm_cmd
        .args([
            "watch",
            "--why",
            "-w",
            "shared/rust/browser-runtime",
            "-w",
            "shared/rust/service-worker",
            "-x",
            "xtask browser-runtime build",
        ])
        .current_dir(&root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    children.push((
        "wasm-watch",
        wasm_cmd
            .spawn()
            .map_err(|e| format!("could not start wasm-watch: {e}"))?,
    ));
    println!("  spawned wasm-watch (browser-runtime + service-worker)");

    println!("hybrid HMR ready — edit shared/rust/dioxus_ui/src/components/stock_data_card.rs and see <500ms patch");
    println!("  urls: http://localhost:3000 (dx HMR) http://localhost:3001 (dx admin) http://localhost:8788/health (workerd) http://localhost:8790 (pay)");
    println!("  tunnel: dev.epsx.io→3000 dev-admin→3001 dev-api→8788 dev-pay→8790 — run cloudflared tunnel separately");

    // Wait for any child to exit — critical services (dx, workerd) kill all, tailwind/wasm 0-exit is non-fatal (tailwind v4 --watch may finish initial build and exit)
    loop {
        let mut exited: Option<(usize, String, std::process::ExitStatus)> = None;
        for (idx, (label, child)) in children.iter_mut().enumerate() {
            if let Some(status) = child
                .try_wait()
                .map_err(|e| format!("could not inspect {label}: {e}"))?
            {
                exited = Some((idx, label.to_string(), status));
                break;
            }
        }
        if let Some((idx, label, status)) = exited {
            let is_non_critical = label.starts_with("tailwind") || label == "wasm-watch";
            if is_non_critical && status.success() {
                println!("{label} exited with {status} (non-critical, continuing)");
                children.remove(idx);
                continue;
            }
            for (j, (_, c)) in children.iter_mut().enumerate() {
                if j != idx {
                    let _ = c.kill();
                }
            }
            return Err(format!("{label} exited with {status}"));
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
}

fn cloudflare_build(flags: &[String]) -> Result<(), String> {
    if !flags.is_empty() {
        return Err("cloudflare build takes no flags".into());
    }
    let root = repo_root()?;
    // Reuse existing browser-runtime build (wasm-bindgen 0.2.123, wasm-opt)
    let browser = crate::node_free::browser_runtime(&["build".to_string()]);
    if let Err(e) = browser {
        return Err(format!("browser-runtime build failed: {e}"));
    }
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--workspace", "--locked", "--release"]);
    cmd.current_dir(&root);
    run_status(&mut cmd, "cargo build --workspace --release (cloudflare)")?;
    println!("cloudflare build: PASS — wrangler configs in apps/*/wrangler.jsonc, assets in dist + target/epsx-browser-runtime");
    Ok(())
}
