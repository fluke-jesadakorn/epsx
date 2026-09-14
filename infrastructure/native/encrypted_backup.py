"""Age-encrypted native backup archives with verified, isolated extraction."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

AGE = "/opt/homebrew/bin/age"


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def verify(directory):
    directory = Path(directory).resolve()
    manifest = json.loads((directory / "manifest.json").read_text())
    expected = {"manifest.json"}
    for entry in manifest["files"]:
        relative = Path(entry["path"])
        path = directory / relative
        if relative.is_absolute() or ".." in relative.parts or path.is_symlink():
            raise RuntimeError("Invalid backup manifest path")
        if entry["path"] in expected or not path.is_file() or digest(path) != entry["sha256"]:
            raise RuntimeError("Backup contents failed checksum verification")
        expected.add(entry["path"])
    actual = {str(p.relative_to(directory)) for p in directory.rglob("*") if p.is_file()}
    if actual != expected or any(p.is_symlink() for p in directory.rglob("*")):
        raise RuntimeError("Backup contains unlisted files or symlinks")
    return manifest


def encrypt(directory, output, recipient):
    directory, output = Path(directory), Path(output)
    verify(directory)
    if output.exists():
        raise RuntimeError("Refusing to replace an encrypted backup")
    partial = output.with_name(output.name + ".partial")
    with partial.open("xb") as target:
        os.chmod(partial, 0o600)
        proc = subprocess.Popen([AGE, "--recipient", recipient], stdin=subprocess.PIPE, stdout=target, stderr=subprocess.PIPE)
        try:
            with tarfile.open(fileobj=proc.stdin, mode="w|gz") as archive:
                for path in sorted(directory.rglob("*")):
                    if path.is_symlink():
                        raise RuntimeError("Symlinks cannot be archived")
                    archive.add(path, arcname=str(path.relative_to(directory)), recursive=False)
            proc.stdin.close()
            error = proc.stderr.read()
            if proc.wait() != 0:
                raise RuntimeError("Age encryption failed")
            target.flush()
            os.fsync(target.fileno())
        except BaseException:
            proc.kill()
            proc.wait()
            raise
        finally:
            proc.stderr.close()
    # Same-filesystem publication after the complete ciphertext is durable.
    os.link(partial, output)
    partial.unlink()
    return digest(output)


def decrypt(archive, identity, output):
    """No overwrite, traversal, symlinks, devices, or trusted archive metadata."""
    archive, identity, output = Path(archive), Path(identity), Path(output)
    if output.exists():
        raise RuntimeError("Choose a new isolated restore directory")
    output.mkdir(mode=0o700, parents=True)
    proc = subprocess.Popen([AGE, "--decrypt", "--identity", str(identity), str(archive)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        seen = set()
        with tarfile.open(fileobj=proc.stdout, mode="r|gz") as contents:
            for member in contents:
                relative = Path(member.name)
                if relative.is_absolute() or ".." in relative.parts or member.name in seen:
                    raise RuntimeError("Unsafe or duplicate archive path")
                seen.add(member.name)
                target = output / relative
                if member.isdir():
                    target.mkdir(mode=0o700, parents=True, exist_ok=True)
                elif member.isfile():
                    target.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                    with target.open("xb") as stream, contents.extractfile(member) as source:
                        os.chmod(target, 0o600)
                        shutil.copyfileobj(source, stream)
                else:
                    raise RuntimeError("Only regular files and directories are permitted")
        # Consume the full authenticated stream; a valid tar header is not
        # enough to accept truncated ciphertext or a failed age process.
        while proc.stdout.read(1024 * 1024):
            pass
        proc.stderr.read()
        if proc.wait() != 0:
            raise RuntimeError("Age decryption/authentication failed")
        return verify(output)
    except BaseException:
        proc.kill()
        proc.wait()
        shutil.rmtree(output)
        raise
    finally:
        proc.stdout.close()
        proc.stderr.close()


def verified_encrypt(directory, output, identity):
    recipient = subprocess.check_output(["/opt/homebrew/bin/age-keygen", "-y", str(identity)], text=True).strip()
    checksum = encrypt(directory, output, recipient)
    with tempfile.TemporaryDirectory(prefix="epsx-backup-verify-", dir=Path(output).parent) as root:
        decrypt(output, identity, Path(root) / "restored")
    return checksum


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("encrypt", "decrypt"))
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--identity", type=Path, required=True)
    args = parser.parse_args()
    os.umask(0o077)
    if args.command == "encrypt":
        checksum = verified_encrypt(args.input, args.output, args.identity)
        print("Encrypted backup and isolated decryption verified; SHA-256:", checksum)
    else:
        decrypt(args.input, args.identity, args.output)
        print("Decrypted into a new directory; all manifest checksums verified")
