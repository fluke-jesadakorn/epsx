import datetime
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest

from encrypted_backup import decrypt, digest, verified_encrypt


class EncryptedBackupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="epsx-backup-test-")
        self.root = Path(self.temp.name)
        self.identity = self.root / "identity.txt"
        subprocess.run(["/opt/homebrew/bin/age-keygen", "-o", str(self.identity)], check=True, capture_output=True)
        self.source = self.root / "source"
        self.source.mkdir()
        (self.source / "data.dump").write_bytes(b"rehearsal database\x00\xff")
        (self.source / "manifest.json").write_text(json.dumps({"files": [{"path": "data.dump", "sha256": digest(self.source / "data.dump")}]}))

    def tearDown(self):
        self.temp.cleanup()

    def test_authenticated_roundtrip_and_no_overwrite(self):
        archive = self.root / "backup.age"
        verified_encrypt(self.source, archive, self.identity)
        target = self.root / "restore"
        decrypt(archive, self.identity, target)
        self.assertEqual((self.source / "data.dump").read_bytes(), (target / "data.dump").read_bytes())
        with self.assertRaises(RuntimeError):
            decrypt(archive, self.identity, target)
        broken = self.root / "truncated.age"
        broken.write_bytes(archive.read_bytes()[:-20])
        with self.assertRaises(Exception):
            decrypt(broken, self.identity, self.root / "bad")
        self.assertFalse((self.root / "bad").exists())

    def test_archive_traversal_and_links_rejected(self):
        recipient = subprocess.check_output(["/opt/homebrew/bin/age-keygen", "-y", str(self.identity)], text=True).strip()
        for index, kind in enumerate(("../outside", "symlink")):
            stream = io.BytesIO()
            with tarfile.open(fileobj=stream, mode="w:gz") as tar:
                item = tarfile.TarInfo(kind)
                if kind == "symlink":
                    item.type = tarfile.SYMTYPE
                    item.linkname = "/etc/passwd"
                tar.addfile(item)
            archive = self.root / f"malicious-{index}.age"
            subprocess.run(["/opt/homebrew/bin/age", "-r", recipient, "-o", str(archive)], input=stream.getvalue(), check=True)
            with self.assertRaises(RuntimeError):
                decrypt(archive, self.identity, self.root / f"reject-{index}")
        self.assertFalse((self.root / "outside").exists())

    def test_retention_keeps_daily_and_weekly_recovery_points(self):
        spec = importlib.util.spec_from_file_location("backup_cycle", Path(__file__).with_name("backup-cycle.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        now = datetime.datetime(2026, 9, 13, 3, tzinfo=datetime.timezone.utc)
        records = [(f"day-{day}", now - datetime.timedelta(days=day)) for day in range(40)]
        records.append(("same-day-older", now - datetime.timedelta(hours=1)))
        keep = module.retention_keep(records)
        self.assertTrue({f"day-{day}" for day in range(7)} <= keep)
        self.assertNotIn("same-day-older", keep)
        self.assertNotIn("day-39", keep)
        self.assertEqual(len({created.isocalendar()[:2] for name, created in records if name in keep}), 4)


if __name__ == "__main__":
    unittest.main()
