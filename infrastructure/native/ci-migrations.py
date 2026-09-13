#!/usr/bin/env python3
"""Exercise all native migration families on empty, local CI shadow databases."""
import os
from pathlib import Path
import re
import subprocess
import tempfile
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[2]
FAMILIES = {
    "core": ("DATABASE_URL", "apps/backend/migrations/core"),
    "analytics": ("ANALYTICS_DATABASE_URL", "apps/backend/migrations/analytics"),
    "payments": ("PAYMENTS_DATABASE_URL", "apps/backend/migrations/payments"),
    "notifications": ("NOTIFICATIONS_DATABASE_URL", "apps/backend/migrations/notifications"),
    "wallet": ("WALLET_DATABASE_URL", "services/wallet/migrations"),
    "pay": ("PAY_SERVICE_DATABASE_URL", "services/pay/migrations"),
    "subscription": ("SUBSCRIPTION_DATABASE_URL", "services/subscription/migrations"),
    "analytics-service": ("ANALYTICS_SERVICE_DATABASE_URL", "services/analytics/migrations"),
}


def main():
    psql = str(Path(os.environ.get("EPSX_PG_BIN", "/usr/bin")) / "psql")
    urls = {}
    for family, (key, _) in FAMILIES.items():
        value = os.environ[key]
        parsed = urlsplit(value)
        if (parsed.scheme not in ("postgres", "postgresql")
                or parsed.hostname not in ("localhost", "127.0.0.1")
                or not re.fullmatch(r"/epsx_[a-z0-9_]+_shadow", parsed.path)):
            raise RuntimeError(f"{family}: requires a local epsx_*_shadow database")
        urls[family] = value

    def sql(url, statement):
        result = subprocess.run(
            [psql, "-X", "-v", "ON_ERROR_STOP=1", "-At", url, "-c", statement],
            check=True, capture_output=True, text=True,
        )
        return result.stdout.strip()

    for url in set(urls.values()):
        if sql(url, "SELECT COUNT(*) FROM pg_tables WHERE schemaname NOT IN ('pg_catalog','information_schema')") != "0":
            raise RuntimeError("Shadow database must be empty before the first migration")

    with tempfile.TemporaryDirectory(prefix="epsx-ci-migrations-") as temporary:
        root = Path(temporary)
        (root / "services").mkdir()
        for family, (_, relative) in FAMILIES.items():
            target = root / (family if family in ("core", "analytics", "payments", "notifications")
                             else "services/" + family.removesuffix("-service"))
            target.symlink_to(ROOT / relative, target_is_directory=True)
        command = [str(ROOT / "target/debug/migrate"), "--root", str(root), "up"]
        subprocess.run(command, check=True, cwd=ROOT)
        ledgers = {}
        ledger_query = "SELECT family,version,checksum,source FROM epsx_schema_migrations ORDER BY family,version"
        for url in set(urls.values()):
            sql(url, "CREATE TABLE migration_sentinel (id integer PRIMARY KEY, marker text NOT NULL); INSERT INTO migration_sentinel VALUES (1, 'preserve-me')")
            ledgers[url] = sql(url, ledger_query)
        subprocess.run(command, check=True, cwd=ROOT)
        for url in set(urls.values()):
            if sql(url, ledger_query) != ledgers[url]:
                raise RuntimeError("Second migration changed the checksum ledger")
            if sql(url, "SELECT marker FROM migration_sentinel WHERE id=1") != "preserve-me":
                raise RuntimeError("Migration sentinel was not preserved")
        for family, (_, relative) in FAMILIES.items():
            directory = ROOT / relative
            expected = sum(1 for path in directory.iterdir()
                           if (path.is_dir() and (path / "up.sql").is_file())
                           or (path.is_file() and path.suffix == ".sql"))
            actual = int(sql(urls[family], f"SELECT COUNT(*) FROM epsx_schema_migrations WHERE family='{family}'"))
            if actual != expected:
                raise RuntimeError(f"{family}: ledger count {actual} != source count {expected}")
            print(f"{family}: {actual} migrations, replay and sentinel PASS")


if __name__ == "__main__":
    main()
