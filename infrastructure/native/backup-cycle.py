#!/usr/bin/env python3
"""Quiesce native system jobs, back up, verify age encryption, and resume jobs.

Install this and its helpers root-owned in /opt/epsx/ops. Configuration is a
root-only JSON file at /etc/epsx/backup/config.json. No checkout env is loaded.
"""
import datetime
import fcntl
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time
import urllib.request

sys.path.insert(0, str(Path(__file__).resolve().parent))
from encrypted_backup import digest, verified_encrypt

ROOT = Path("/var/backups/epsx")
STATE = ROOT / "cycle-state.json"
CONFIG = Path("/etc/epsx/backup/config.json")
STOP_ORDER = ("tunnel", "bff-frontend", "bff-admin", "bff-pay", "epsx",
              "analytics", "notification", "subscription", "pay-service", "wallet", "minio")
FAMILIES = ("DATABASE_URL", "ANALYTICS_DATABASE_URL", "PAYMENTS_DATABASE_URL",
            "NOTIFICATIONS_DATABASE_URL", "WALLET_DATABASE_URL", "PAY_SERVICE_DATABASE_URL",
            "SUBSCRIPTION_DATABASE_URL", "ANALYTICS_SERVICE_DATABASE_URL")


def write_json(path, value):
    temporary = path.with_suffix(".tmp")
    with temporary.open("w") as stream:
        json.dump(value, stream, indent=2)
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temporary, path)


def loaded(name):
    return subprocess.run(["/bin/launchctl", "print", "system/com.epsx.native." + name], capture_output=True).returncode == 0


def restore(names):
    failures = []
    for name in reversed(STOP_ORDER):
        if name not in names or loaded(name):
            continue
        # Do not expose traffic until the backend has recovered its dependencies.
        if name == "tunnel" and "epsx" in names:
            for _ in range(120):
                try:
                    with urllib.request.urlopen("http://127.0.0.1:9180/ready", timeout=2) as response:
                        if response.status == 200:
                            break
                except Exception:
                    time.sleep(1)
            else:
                failures.append("tunnel (backend readiness failed)")
                continue
        result = subprocess.run(["/bin/launchctl", "bootstrap", "system", "/Library/LaunchDaemons/com.epsx.native." + name + ".plist"], capture_output=True)
        if result.returncode:
            failures.append(name)
    if failures:
        raise RuntimeError("Could not resume native jobs: " + ", ".join(failures))


def retention_keep(records):
    """Seven distinct daily points plus four distinct ISO weekly points."""
    keep, days, weeks = set(), set(), set()
    for name, created in sorted(records, key=lambda row: row[1], reverse=True):
        day, week = created.date(), created.isocalendar()[:2]
        if day not in days and len(days) < 7:
            keep.add(name)
            days.add(day)
        if week not in weeks and len(weeks) < 4:
            keep.add(name)
            weeks.add(week)
    return keep


def prune():
    candidates = []
    for metadata in ROOT.glob("native-*.json"):
        if not re.fullmatch(r"native-\d{8}T\d{6}Z\.json", metadata.name):
            continue
        data = json.loads(metadata.read_text())
        archive = metadata.with_suffix(".age")
        if data.get("verified") is True and archive.is_file() and digest(archive) == data["sha256"]:
            candidates.append((archive.name, datetime.datetime.fromisoformat(data["created_at"])))
    keep = retention_keep(candidates)
    for name, _ in candidates:
        if name not in keep:
            (ROOT / name).unlink()
            (ROOT / name).with_suffix(".json").unlink()


def main():
    if os.geteuid() != 0:
        raise SystemExit("backup-cycle requires root")
    os.umask(0o077)
    if CONFIG.stat().st_uid != 0 or CONFIG.stat().st_mode & 0o077:
        raise RuntimeError("Backup configuration must be root-owned mode 600")
    config = json.loads(CONFIG.read_text())
    environment = {k: v for k, v in config["environment"].items() if k in FAMILIES}
    if set(environment) != set(FAMILIES):
        raise RuntimeError("All eight database families must be configured")
    environment.update(EPSX_PG_BIN="/opt/homebrew/opt/postgresql@17/bin",
        EPSX_KEYS_DIR="/etc/epsx/keys", EPSX_CONFIG_DIR="/etc/epsx",
        EPSX_MINIO_DATA_DIR="/var/db/epsx/minio", PATH="/usr/bin:/bin")
    identity = Path("/etc/epsx/backup/identity.txt")
    if identity.stat().st_uid != 0 or identity.stat().st_mode & 0o077:
        raise RuntimeError("Backup identity must be root-owned mode 600")
    ROOT.mkdir(mode=0o700, parents=True, exist_ok=True)
    with (ROOT / "cycle.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if STATE.exists():
            restore(json.loads(STATE.read_text())["jobs"])
            STATE.unlink()
            raise RuntimeError("Recovered jobs from interrupted backup; inspect before retry")
        names = [name for name in STOP_ORDER if loaded(name)]
        if "epsx" not in names or "minio" not in names:
            raise RuntimeError("Native backend and MinIO jobs must be installed before scheduled backups")
        created = datetime.datetime.now(datetime.timezone.utc)
        stamp = created.strftime("%Y%m%dT%H%M%SZ")
        archive = ROOT / ("native-" + stamp + ".age")
        write_json(STATE, {"jobs": names, "started_at": created.isoformat()})

        def interrupted(signum, frame):
            raise RuntimeError("Backup interrupted by signal " + str(signum))
        signal.signal(signal.SIGTERM, interrupted)
        signal.signal(signal.SIGINT, interrupted)
        try:
            for name in names:
                subprocess.run(["/bin/launchctl", "bootout", "system/com.epsx.native." + name], check=True, timeout=180)
            with tempfile.TemporaryDirectory(prefix=".native-snapshot-", dir=ROOT) as temporary:
                snapshot = Path(temporary) / "snapshot"
                subprocess.run([sys.executable, str(Path(__file__).with_name("backup.py")), "--output", str(snapshot), "--writers-quiesced", "--minio-stopped"], env=environment, check=True)
                checksum = verified_encrypt(snapshot, archive, identity)
            write_json(archive.with_suffix(".json"), {"created_at": created.isoformat(), "sha256": checksum, "verified": True})
        finally:
            restore(names)
            STATE.unlink()
        prune()
        print("Encrypted native backup verified; jobs resumed:", archive)


if __name__ == "__main__":
    main()
