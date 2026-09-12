#!/usr/bin/env python3
"""Install isolated native storage in the system launchd domain.

Run render first for review. install-storage requires root and never starts
application releases, imports production data, or changes Tunnel/DNS routes.
"""
import argparse
import datetime
import grp
import hashlib
import json
import os
from pathlib import Path
import plistlib
import pwd
import secrets
import socket
import subprocess
import urllib.parse

USER = "_epsx"
CONFIG = Path("/etc/epsx")
DATA = Path("/var/db/epsx")
PREFIX = Path("/opt/epsx")
PG = Path("/opt/homebrew/opt/postgresql@17/bin")
LABELS = ("postgresql", "redis", "minio")

CANDIDATE_FAMILIES = {
    "DATABASE_URL": "core", "ANALYTICS_DATABASE_URL": "analytics",
    "PAYMENTS_DATABASE_URL": "payments", "NOTIFICATIONS_DATABASE_URL": "notifications",
    "WALLET_DATABASE_URL": "wallet", "PAY_SERVICE_DATABASE_URL": "pay",
    "SUBSCRIPTION_DATABASE_URL": "subscription", "ANALYTICS_SERVICE_DATABASE_URL": "analytics",
}


def check_candidate_snapshot(snapshot):
    snapshot = snapshot.resolve()
    manifest = json.loads((snapshot / "manifest.json").read_text())
    if manifest.get("pg_dump_major") != 17 or set(manifest["database_families"]) != set(CANDIDATE_FAMILIES):
        raise RuntimeError("A verified PostgreSQL 17 snapshot of all eight families is required")
    listed = set()
    for entry in manifest["files"]:
        relative = Path(entry["path"])
        path = snapshot / relative
        if relative.is_absolute() or ".." in relative.parts or not path.resolve().is_relative_to(snapshot) or path.is_symlink():
            raise RuntimeError("Invalid snapshot path")
        if entry["path"] in listed:
            raise RuntimeError("Duplicate snapshot manifest entry")
        with path.open("rb") as stream:
            checksum = hashlib.file_digest(stream, "sha256").hexdigest()
        if checksum != entry["sha256"]:
            raise RuntimeError("Snapshot checksum mismatch")
        listed.add(entry["path"])
    if any(key + ".dump" not in listed for key in CANDIDATE_FAMILIES):
        raise RuntimeError("A required database dump is missing from the manifest")
    return manifest


def import_candidate(snapshot):
    """Create new isolated DBs; never replace a database, app config, or route."""
    if os.geteuid() != 0:
        raise SystemExit("import-candidate requires root")
    os.umask(0o077)
    manifest = check_candidate_snapshot(snapshot)
    identity()
    password = (CONFIG / "postgres-owner.pass").read_text().strip()
    env = {"PATH": "/usr/bin:/bin", "LC_ALL": "C", "LANG": "C",
           "PGHOST": "127.0.0.1", "PGPORT": "55433", "PGUSER": "epsx_owner",
           "PGPASSWORD": password, "PGDATABASE": "postgres"}
    major = subprocess.check_output([str(PG / "psql"), "-XAt", "-c", "SHOW server_version_num"], env=env, text=True).strip()
    if int(major) // 10000 != 17:
        raise RuntimeError("Candidate destination must be the isolated PostgreSQL 17 server")
    stamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y%m%d%H%M%S")
    candidates = CONFIG / "candidates"
    candidates.mkdir(mode=0o700, exist_ok=True)
    record = candidates / (stamp + ".json")
    if record.exists():
        raise RuntimeError("Candidate record already exists")
    result = {"id": stamp, "source": str(snapshot.resolve()), "source_created_at": manifest["created_at"],
              "status": "importing", "environment": {}, "databases": []}
    record.write_text(json.dumps(result, indent=2))
    databases = {}
    for key, family in CANDIDATE_FAMILIES.items():
        if family in databases:
            result["environment"][key] = databases[family]
            continue
        name = f"epsx_candidate_{stamp}_{family}"
        role = f"epsx_candidate_{stamp}_{family}"
        secret = secrets.token_urlsafe(48)
        # Identifiers and passwords are generated here, never interpolated from
        # snapshot content. Role passwords never appear in process arguments.
        sql = f'CREATE ROLE "{role}" LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION PASSWORD \'{secret}\';'
        subprocess.run([str(PG / "psql"), "-Xq", "-v", "ON_ERROR_STOP=1"], input=sql, env=env, text=True, check=True, capture_output=True)
        subprocess.run([str(PG / "createdb"), "--owner", role, name], env=env, check=True, capture_output=True)
        url = f"postgresql://{role}:{urllib.parse.quote(secret, safe='')}@127.0.0.1:55433/{name}"
        result["environment"][key] = url
        result["databases"].append({"family": family, "name": name, "role": role, "status": "restoring"})
        record.write_text(json.dumps(result, indent=2))
        subprocess.run([str(PG / "pg_restore"), "--exit-on-error", "--single-transaction", "--no-owner", "--no-acl", "--role", role,
                        "--dbname", name, str(snapshot / (key + ".dump"))], env=env, check=True, capture_output=True)
        # Verify actual password login as the non-superuser application role.
        role_env = {**env, "PGDATABASE": name, "PGUSER": role, "PGPASSWORD": secret}
        check = subprocess.check_output([str(PG / "psql"), "-XAt", "-v", "ON_ERROR_STOP=1", "-c",
            "SELECT current_user = pg_get_userbyid(datdba) AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole FROM pg_database JOIN pg_roles ON rolname=current_user WHERE datname=current_database()"], env=role_env, text=True).strip()
        if check != "t":
            raise RuntimeError("Candidate application role verification failed")
        result["databases"][-1]["status"] = "restored"
        databases[family] = url
        record.write_text(json.dumps(result, indent=2))
        print("Candidate database restored:", name, flush=True)
    result["status"] = "ready_for_private_rehearsal"
    record.write_text(json.dumps(result, indent=2))
    print("Private candidate configuration:", record)
    print("No application service, production database, or public route was replaced.")


def run(args, **kwargs):
    return subprocess.run([str(a) for a in args], check=True, **kwargs)


def storage_plists():
    commands = {
        "postgresql": [str(PG / "postgres"), "-D", str(DATA / "postgresql")],
        "redis": ["/opt/homebrew/bin/redis-server", str(CONFIG / "redis.conf")],
        "minio": ["/bin/bash", str(PREFIX / "ops/run-minio.sh")],
    }
    return {name: {
        "Label": f"com.epsx.native.{name}", "UserName": USER,
        "GroupName": USER, "ProgramArguments": command,
        "WorkingDirectory": str(DATA), "RunAtLoad": True, "KeepAlive": True,
        "ThrottleInterval": 10, "ExitTimeOut": 120,
        "Umask": 0o077,
        "EnvironmentVariables": {"PATH": "/opt/homebrew/bin:/usr/bin:/bin", "HOME": str(DATA),
            **({"LC_ALL": "C", "LANG": "C"} if name == "postgresql" else {})},
        "StandardOutPath": str(CONFIG / "logs" / f"{name}.log"),
        "StandardErrorPath": str(CONFIG / "logs" / f"{name}.error.log"),
    } for name, command in commands.items()}


def render(output):
    output.mkdir(parents=True, exist_ok=False)
    for name, definition in storage_plists().items():
        (output / f"com.epsx.native.{name}.plist").write_bytes(plistlib.dumps(definition))


def identity():
    try:
        user = pwd.getpwnam(USER)
        group = grp.getgrnam(USER)
        if user.pw_uid >= 500 or user.pw_gid != group.gr_gid or user.pw_dir != str(DATA) or user.pw_shell != "/usr/bin/false":
            raise RuntimeError("Existing _epsx account is not the expected service identity")
        return user.pw_uid, group.gr_gid
    except KeyError:
        used = {p.pw_uid for p in pwd.getpwall()} | {g.gr_gid for g in grp.getgrall()}
        uid = next(i for i in range(499, 299, -1) if i not in used)
        # Refuse partial pre-existing identities instead of overwriting them.
        for collection in ("Users", "Groups"):
            r = subprocess.run(["dscl", ".", "-read", f"/{collection}/{USER}"], capture_output=True)
            if r.returncode == 0:
                raise RuntimeError("Partial _epsx identity exists; inspect before installation")
        for collection, attributes in (
            ("Groups", {"PrimaryGroupID":str(uid), "RealName":"EPSX native services"}),
            ("Users", {"UniqueID":str(uid), "PrimaryGroupID":str(uid), "NFSHomeDirectory":str(DATA),
                       "UserShell":"/usr/bin/false", "RealName":"EPSX native services", "IsHidden":"1"}),
        ):
            path = f"/{collection}/{USER}"
            run(["dscl", ".", "-create", path])
            for key, value in attributes.items():
                run(["dscl", ".", "-create", path, key, value])
        return uid, uid


def private_file(path, text, uid, gid):
    if path.exists():
        raise RuntimeError(f"Refusing to overwrite configuration: {path}")
    with path.open("x") as stream:
        stream.write(text)
    os.chmod(path, 0o600)
    os.chown(path, uid, gid)


def install_storage():
    if os.geteuid() != 0:
        raise SystemExit("install-storage requires root")
    os.umask(0o077)
    for binary in (PG / "postgres", PG / "initdb", Path("/opt/homebrew/bin/redis-server"), Path("/opt/homebrew/bin/minio")):
        if not binary.is_file() or not os.access(binary, os.X_OK):
            raise RuntimeError(f"Missing executable: {binary}")
    if not (CONFIG / "storage-installed").exists():
        for port in (55433, 6380, 9100, 9101):
            with socket.socket() as probe:
                probe.bind(("127.0.0.1", port))
    uid, gid = identity()
    for directory, owner, mode in (
        (PREFIX, 0, 0o755), (PREFIX / "ops", 0, 0o755), (PREFIX / "releases", 0, 0o755),
        (CONFIG, 0, 0o750), (CONFIG / "logs", uid, 0o750), (CONFIG / "keys", 0, 0o750),
        (DATA, uid, 0o750), (DATA / "redis", uid, 0o700), (DATA / "minio", uid, 0o700),
        (DATA / "run", uid, 0o700), (Path("/var/backups/epsx"), 0, 0o700),
    ):
        directory.mkdir(parents=True, exist_ok=True)
        os.chown(directory, owner, gid)
        os.chmod(directory, mode)
    marker = CONFIG / "storage-installed"
    if not marker.exists():
        if (DATA / "postgresql").exists() or any((CONFIG / name).exists() for name in ("postgres-owner.pass", "redis.conf", "minio.env")):
            raise RuntimeError("Partial storage installation exists; reconcile before retry")
        pg_password, redis_password, minio_password = (secrets.token_urlsafe(48) for _ in range(3))
        private_file(CONFIG / "postgres-owner.pass", pg_password + "\n", uid, gid)
        run(["sudo", "-u", USER, PG / "initdb", "--pgdata", DATA / "postgresql",
             "--username", "epsx_owner", "--auth-local=scram-sha-256", "--auth-host=scram-sha-256",
             "--encoding=UTF8", "--locale=C", "--pwfile", CONFIG / "postgres-owner.pass"], stdout=subprocess.DEVNULL)
        with (DATA / "postgresql/postgresql.conf").open("a") as stream:
            stream.write("\n# EPSX isolated native instance\nlisten_addresses='127.0.0.1'\nport=55433\nunix_socket_directories='/var/db/epsx/run'\npassword_encryption='scram-sha-256'\nmax_connections=100\nshared_buffers='256MB'\nlog_connections=off\nlog_disconnections=off\n")
        private_file(CONFIG / "redis.conf", f"bind 127.0.0.1\nport 6380\nprotected-mode yes\nrequirepass {redis_password}\nappendonly yes\nappendfsync everysec\ndir /var/db/epsx/redis\nlogfile \"\"\n", uid, gid)
        private_file(CONFIG / "minio.env", f"MINIO_ROOT_USER=epsx_native\nMINIO_ROOT_PASSWORD={minio_password}\nMINIO_BROWSER_REDIRECT_URL=http://127.0.0.1:9101\n", uid, gid)
        private_file(CONFIG / "storage.env", f"EPSX_POSTGRES_OWNER_PASSWORD={pg_password}\nEPSX_REDIS_PASSWORD={redis_password}\nEPSX_MINIO_ROOT_USER=epsx_native\nEPSX_MINIO_ROOT_PASSWORD={minio_password}\n", 0, gid)
        marker.write_text("PostgreSQL 55433; Redis 6380; MinIO 9100/9101. No public routes.\n")
        os.chmod(marker, 0o600)
    wrapper = PREFIX / "ops/run-minio.sh"
    contents = '#!/bin/bash\nset -euo pipefail\numask 077\nset -a\n. /etc/epsx/minio.env\nset +a\nexec /opt/homebrew/bin/minio server /var/db/epsx/minio --address 127.0.0.1:9100 --console-address 127.0.0.1:9101\n'
    if wrapper.exists() and wrapper.read_text() != contents:
        raise RuntimeError("Existing MinIO wrapper differs; review before replacing")
    wrapper.write_text(contents)
    os.chown(wrapper, 0, 0)
    os.chmod(wrapper, 0o755)
    for name, definition in storage_plists().items():
        target = Path("/Library/LaunchDaemons") / f"com.epsx.native.{name}.plist"
        if target.exists() and plistlib.loads(target.read_bytes()) != definition:
            raise RuntimeError(f"Existing system job differs: {target}")
        target.write_bytes(plistlib.dumps(definition))
        os.chown(target, 0, 0)
        os.chmod(target, 0o644)
        loaded = subprocess.run(["launchctl", "print", f"system/com.epsx.native.{name}"], capture_output=True).returncode == 0
        if not loaded:
            run(["launchctl", "bootstrap", "system", target])
        print(f"System storage job installed: com.epsx.native.{name}")


def repair_postgres_locale():
    if os.geteuid() != 0:
        raise SystemExit("repair-postgres-locale requires root")
    target = Path("/Library/LaunchDaemons/com.epsx.native.postgresql.plist")
    current = plistlib.loads(target.read_bytes())
    expected = storage_plists()["postgresql"]
    previous = plistlib.loads(plistlib.dumps(expected))
    previous["EnvironmentVariables"].pop("LC_ALL")
    previous["EnvironmentVariables"].pop("LANG")
    if current not in (expected, previous):
        raise RuntimeError("PostgreSQL job differs beyond locale; inspect before changing")
    backup = CONFIG / "postgresql-launchd-before-locale.plist"
    if not backup.exists():
        backup.write_bytes(target.read_bytes())
        os.chmod(backup, 0o600)
    subprocess.run(["launchctl", "bootout", "system", str(target)], capture_output=True)
    target.write_bytes(plistlib.dumps(expected))
    os.chown(target, 0, 0)
    os.chmod(target, 0o644)
    run(["launchctl", "bootstrap", "system", target])
    print("PostgreSQL system job reloaded with LC_ALL=C and LANG=C")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("render").add_argument("--output", type=Path, required=True)
    sub.add_parser("install-storage")
    sub.add_parser("repair-postgres-locale")
    candidate = sub.add_parser("import-candidate")
    candidate.add_argument("--snapshot", type=Path, required=True)
    candidate.add_argument("--check-only", action="store_true")
    args = parser.parse_args()
    if args.command == "render":
        render(args.output)
        print(f"Rendered storage LaunchDaemons: {args.output}")
    elif args.command == "install-storage":
        install_storage()
    elif args.command == "repair-postgres-locale":
        repair_postgres_locale()
    elif args.check_only:
        check_candidate_snapshot(args.snapshot)
        print("Candidate snapshot manifest and all eight database dumps verified")
    else:
        import_candidate(args.snapshot)


if __name__ == "__main__":
    main()
