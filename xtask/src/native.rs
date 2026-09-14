//! Native release tooling. Packaging/rendering never starts services or changes production.
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
const BINARIES: &[(&str, &str)] = &[
    ("epsx", "epsx"),
    ("epsx", "migrate"),
    ("epsx-frontend", "bff-frontend"),
    ("epsx-admin", "bff-admin"),
    ("epsx-pay-bff", "bff-pay"),
    ("epsx-wallet", "wallet"),
    ("epsx-pay-svc", "pay-service"),
    ("epsx-subscription", "subscription"),
    ("epsx-notification", "notification"),
    ("epsx-analytics", "analytics"),
];
const FULLSTACK_APPS: &[(&str, &str, &str)] = &[
    ("frontend", "epsx-frontend", "dx-frontend"),
    ("admin", "epsx-admin", "dx-admin"),
    ("pay", "epsx-pay-bff", "epsx-pay"),
];
pub fn run(flags: &[String]) -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    match flags.first().map(String::as_str) {
        Some("package") => package(&root, &flags[1..]),
        Some("launchd") => launchd(&flags[1..]),
        _ => Err("native package --output <new-directory> | native launchd --release <current-link> --config <directory> --output <new-directory> --user <user>".into()),
    }
}
fn value<'a>(flags: &'a [String], key: &str) -> Result<&'a str, String> {
    flags
        .windows(2)
        .find(|w| w[0] == key)
        .map(|w| w[1].as_str())
        .ok_or_else(|| format!("{key} required"))
}
fn io<T>(result: std::io::Result<T>) -> Result<T, String> {
    result.map_err(|e| e.to_string())
}
fn copy_tree(source: &Path, target: &Path) -> Result<(), String> {
    io(fs::create_dir_all(target))?;
    for entry in io(fs::read_dir(source))? {
        let entry = io(entry)?;
        if entry.file_name() == "__pycache__" {
            continue;
        }
        let kind = io(entry.file_type())?;
        if kind.is_symlink() {
            return Err(format!(
                "symlink not permitted in release input: {}",
                entry.path().display()
            ));
        }
        if kind.is_dir() {
            copy_tree(&entry.path(), &target.join(entry.file_name()))?;
        } else {
            io(fs::copy(entry.path(), target.join(entry.file_name())))?;
        }
    }
    Ok(())
}
fn manifest(
    directory: &Path,
    root: &Path,
    files: &mut Vec<serde_json::Value>,
) -> Result<(), String> {
    let mut entries = io(fs::read_dir(directory))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        if io(entry.file_type())?.is_dir() {
            manifest(&entry.path(), root, files)?;
        } else {
            files.push(serde_json::json!({"path":entry.path().strip_prefix(root).unwrap().to_string_lossy(),"sha256":format!("{:x}",Sha256::digest(io(fs::read(entry.path()))?))}));
        }
    }
    Ok(())
}
fn has_hydration_wasm(directory: &Path) -> Result<bool, String> {
    for entry in io(fs::read_dir(directory))? {
        let entry = io(entry)?;
        let kind = io(entry.file_type())?;
        if kind.is_symlink() {
            return Err("symlink in Fullstack assets".into());
        }
        if kind.is_dir() && has_hydration_wasm(&entry.path())? {
            return Ok(true);
        }
        if kind.is_file()
            && entry.path().extension().is_some_and(|ext| ext == "wasm")
            && io(fs::read(entry.path()))?.starts_with(b"\0asm")
        {
            return Ok(true);
        }
    }
    Ok(false)
}
fn package(root: &Path, flags: &[String]) -> Result<(), String> {
    let output = PathBuf::from(value(flags, "--output")?);
    if output.exists() {
        return Err("output already exists; choose a new release directory".into());
    }
    crate::node_free::build_service_worker(root)?;
    let mut command = Command::new("cargo");
    command.current_dir(root).args([
        "build",
        "--release",
        "--locked",
        "--features",
        "epsx/cli-tools",
    ]);
    for (package, binary) in BINARIES {
        command.args(["-p", package, "--bin", binary]);
    }
    if !io(command.status())?.success() {
        return Err("native release build failed; no package produced".into());
    }
    let metadata = io(Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output())?;
    if !metadata.status.success() {
        return Err("cargo metadata failed".into());
    }
    let metadata: serde_json::Value =
        serde_json::from_slice(&metadata.stdout).map_err(|e| e.to_string())?;
    let target = PathBuf::from(
        metadata["target_directory"]
            .as_str()
            .ok_or("missing target directory")?,
    );
    // Each renderer must have its own feature graph. Passing --features web to
    // a fullstack build also enables it on the server and breaks hydration.
    for (app, package, binary) in FULLSTACK_APPS {
        let status = io(Command::new("dx")
            .current_dir(root)
            .args([
                "build",
                "--fullstack",
                "true",
                "--release",
                "--web",
                "--no-default-features",
                "--locked",
                "--force-sequential",
                "true",
                "--package",
                package,
                "--bin",
                binary,
            ])
            .status())?;
        if !status.success() {
            return Err(format!(
                "{app} Fullstack release build failed; no package produced"
            ));
        }
        let public = target.join("dx").join(binary).join("release/web/public");
        if !public.join("index.html").is_file() || !has_hydration_wasm(&public)? {
            return Err(format!(
                "{app} Fullstack release is missing hydration assets"
            ));
        }
    }
    io(fs::create_dir_all(output.join("bin")))?;
    for (_, binary) in BINARIES {
        io(fs::copy(
            target.join("release").join(binary),
            output.join("bin").join(binary),
        ))?;
    }
    copy_tree(
        &root.join("target/epsx-service-worker"),
        &output.join("runtime"),
    )?;
    for app in ["frontend", "admin", "pay"] {
        let source = root.join(format!("apps/{app}/public"));
        let source = if app == "pay" && !source.is_dir() {
            root.join("apps/frontend/public")
        } else {
            source
        };
        copy_tree(&source, &output.join(format!("public/{app}")))?;
    }
    for (app, _, binary) in FULLSTACK_APPS {
        copy_tree(
            &target.join("dx").join(binary).join("release/web/public"),
            &output.join(format!("fullstack/{app}/public")),
        )?;
    }
    for family in ["core", "analytics", "payments", "notifications"] {
        copy_tree(
            &root.join(format!("apps/backend/migrations/{family}")),
            &output.join(format!("migrations/{family}")),
        )?;
    }
    for family in ["wallet", "pay", "subscription", "analytics"] {
        copy_tree(
            &root.join(format!("services/{family}/migrations")),
            &output.join(format!("migrations/services/{family}")),
        )?;
    }
    copy_tree(&root.join("infrastructure/native"), &output.join("ops"))?;
    copy_tree(&root.join("docs/pay"), &output.join("docs/pay"))?;
    let mut files = Vec::new();
    manifest(&output, &output, &mut files)?;
    let version = io(Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output())?;
    let status = io(Command::new("git")
        .current_dir(root)
        .args(["status", "--porcelain"])
        .output())?;
    io(fs::write(output.join("manifest.json"), serde_json::to_vec_pretty(&serde_json::json!({"format":1,"commit":String::from_utf8_lossy(&version.stdout).trim(),"working_tree_dirty":!status.stdout.is_empty(),"built_at":chrono::Utc::now(),"architecture":env::consts::ARCH,"files":files})).map_err(|e| e.to_string())?))?;
    println!(
        "Release packaged at {}. No migrations or deployment performed.",
        output.display()
    );
    Ok(())
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn launchd(flags: &[String]) -> Result<(), String> {
    let release = value(flags, "--release")?;
    let config = value(flags, "--config")?;
    let output = Path::new(value(flags, "--output")?);
    let user = value(flags, "--user")?;
    let tunnel = match (
        value(flags, "--cloudflared"),
        value(flags, "--tunnel-config"),
    ) {
        (Ok(binary), Ok(config))
            if Path::new(binary).is_absolute() && Path::new(config).is_absolute() =>
        {
            Some((binary, config))
        }
        (Err(_), Err(_)) => None,
        _ => return Err(
            "tunnel rendering requires both --cloudflared and --tunnel-config as absolute paths"
                .into(),
        ),
    };
    if !Path::new(release).is_absolute() || !Path::new(config).is_absolute() {
        return Err("release and config must be absolute paths".into());
    }
    io(fs::create_dir(output))?;
    if let Some((cloudflared, tunnel_config)) = tunnel {
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>com.epsx.native.tunnel</string>
<key>UserName</key><string>{user}</string>
<key>ProgramArguments</key><array><string>{cloudflared}</string><string>--no-autoupdate</string><string>tunnel</string><string>--config</string><string>{tunnel_config}</string><string>run</string></array>
<key>RunAtLoad</key><true/><key>KeepAlive</key><true/><key>ThrottleInterval</key><integer>10</integer>
<key>StandardOutPath</key><string>{config}/logs/tunnel.log</string><key>StandardErrorPath</key><string>{config}/logs/tunnel.error.log</string>
</dict></plist>
"#,
            user = xml(user),
            cloudflared = xml(cloudflared),
            tunnel_config = xml(tunnel_config),
            config = xml(config)
        );
        io(fs::write(output.join("com.epsx.native.tunnel.plist"), body))?;
    }
    for (_, binary) in BINARIES.iter().filter(|(_, b)| *b != "migrate") {
        let body = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>com.epsx.native.{binary}</string>
<key>UserName</key><string>{user}</string>
<key>ProgramArguments</key><array><string>/bin/bash</string><string>{release}/ops/run-service.sh</string><string>{binary}</string><string>{config}</string></array>
<key>RunAtLoad</key><true/><key>KeepAlive</key><true/><key>ThrottleInterval</key><integer>10</integer>
<key>StandardOutPath</key><string>{config}/logs/{binary}.log</string><key>StandardErrorPath</key><string>{config}/logs/{binary}.error.log</string>
</dict></plist>
"#,
            user = xml(user),
            release = xml(release),
            config = xml(config)
        );
        io(fs::write(
            output.join(format!("com.epsx.native.{binary}.plist")),
            body,
        ))?;
    }
    println!(
        "Rendered launch daemons at {}. Not installed or started.",
        output.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hydration_validation_accepts_hashed_release_assets_but_not_empty_directories() {
        let root = env::temp_dir().join(format!("epsx-native-assets-{}", std::process::id()));
        fs::create_dir_all(root.join("assets")).unwrap();
        assert!(!has_hydration_wasm(&root).unwrap());
        fs::write(root.join("assets/app-hash.wasm"), b"\0asm\x01\0\0\0").unwrap();
        assert!(has_hydration_wasm(&root).unwrap());
        fs::remove_dir_all(root).unwrap();
    }
}
