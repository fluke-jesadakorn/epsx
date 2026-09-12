#!/usr/bin/env python3
"""One dev worker: recovery rebuilds and small, DWARF-free DX bundles."""
import os
import pathlib
import signal
import subprocess
import tempfile
import time

REPO = pathlib.Path(os.environ.get("EPSX_DEV_CHECKOUT", pathlib.Path(__file__).resolve().parents[2]))
BUNDLES = ("dx-frontend", "dx-admin", "epsx-pay")


def strip_dwarf(data):
    """Remove only optional .debug_* custom sections; retain code and HMR data."""
    if data[:8] != b"\0asm\x01\0\0\0":
        raise ValueError("Not a complete WebAssembly v1 module")
    result = bytearray(data[:8])
    offset = 8

    def leb(index):
        value = 0
        for shift in range(0, 35, 7):
            if index >= len(data): raise ValueError("Incomplete section size")
            byte = data[index]; index += 1
            value |= (byte & 127) << shift
            if byte < 128: return value, index
        raise ValueError("Invalid section size")

    while offset < len(data):
        start = offset
        kind = data[offset]; offset += 1
        size, offset = leb(offset)
        end = offset + size
        if end > len(data): raise ValueError("Incomplete section")
        remove = False
        if kind == 0:
            length, name_start = leb(offset)
            if name_start + length > end: raise ValueError("Incomplete custom section name")
            remove = data[name_start:name_start + length].startswith(b".debug_")
        if not remove: result.extend(data[start:end])
        offset = end
    return bytes(result)


def compact(path):
    before = path.stat()
    source = path.read_bytes()
    stripped = strip_dwarf(source)
    if len(source) == len(stripped): return
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(dir=path.parent, prefix=".dev-wasm-", delete=False) as output:
            temporary = pathlib.Path(output.name)
            output.write(stripped)
        after = path.stat()
        # A newer build may have replaced this file while we were reading it.
        if (before.st_ino, before.st_mtime_ns, before.st_size) != (after.st_ino, after.st_mtime_ns, after.st_size):
            return
        temporary.chmod(before.st_mode & 0o777)
        os.replace(temporary, path)
        print(f"{path.name}: {len(source)//1024//1024} -> {len(stripped)//1024//1024} MiB (DWARF removed)", flush=True)
    finally:
        if temporary is not None: temporary.unlink(missing_ok=True)


def main():
    child = subprocess.Popen(["cargo", "watch", "--why", "--watch", "shared/rust/service-worker",
                              "--shell", "cargo xtask browser-runtime build"], cwd=REPO,
                             start_new_session=True)
    stopping = False
    def stop(*_):
        nonlocal stopping
        stopping = True
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    seen = {}
    try:
        while not stopping:
            if child.poll() is not None: raise SystemExit(child.returncode or 1)
            for bundle in BUNDLES:
                directory = REPO / "target/dx" / bundle / "debug/web/public/wasm"
                for path in directory.glob("*.wasm"):
                    try:
                        stat = path.stat()
                        stamp = (stat.st_ino, stat.st_mtime_ns, stat.st_size)
                        if seen.get(path) == stamp: continue
                        compact(path)
                        stat = path.stat()
                        seen[path] = (stat.st_ino, stat.st_mtime_ns, stat.st_size)
                    except (OSError, ValueError):
                        # DX may still be writing or replacing its bundle. Retry.
                        pass
            time.sleep(2)
    finally:
        if child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
            try: child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()


if __name__ == "__main__":
    main()
