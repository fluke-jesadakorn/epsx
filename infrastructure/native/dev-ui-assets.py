#!/usr/bin/env python3
"""One dev worker: recovery rebuilds and small, DWARF-free DX bundles."""
import os
import hashlib
import json
import pathlib
import signal
import subprocess
import tempfile
import time

REPO = pathlib.Path(os.environ.get("EPSX_DEV_CHECKOUT", pathlib.Path(__file__).resolve().parents[2]))
BUNDLES = ("dx-frontend", "dx-admin", "epsx-pay")
APPS = dict(zip(BUNDLES, ("frontend", "admin", "pay")))


def stamp(path):
    stat = path.stat()
    return stat.st_ino, stat.st_mtime_ns, stat.st_size


def write_changed(path, data, expected=None):
    if path.exists() and path.read_bytes() == data:
        return
    with tempfile.NamedTemporaryFile(dir=path.parent, prefix='.dev-asset-', delete=False) as output:
        output.write(data)
        temporary = pathlib.Path(output.name)
    try:
        if expected is not None and stamp(path) != expected:
            return
        temporary.chmod(0o644)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def sync_live_css(repo, bundle):
    directory = repo/'target/dx'/bundle/'debug/web/public/wasm'
    bootstrap = directory/f'{bundle}.js'
    if not bootstrap.exists():
        return
    runtime = (repo/'infrastructure/native/dev-live-css.js').read_bytes()
    write_changed(directory/'epsx-dev-live-css.js', runtime)
    styles = {}
    source = repo/'apps'/APPS[bundle]/'public'
    for path in source.rglob('*.css'):
        relative = path.relative_to(source).as_posix()
        css = path.read_bytes()
        version = hashlib.sha256(css).hexdigest()[:16]
        name = 'epsx-dev-css-' + hashlib.sha256(relative.encode()).hexdigest()[:16] + '.css'
        write_changed(directory/name, css)
        for prefix in ('/', '/public/'):
            styles[prefix + relative] = {'file': './' + name, 'version': version}
    revision = hashlib.sha256(json.dumps(styles, sort_keys=True).encode()).hexdigest()
    write_changed(directory/'epsx-dev-live-css.json', json.dumps({'revision': revision, 'styles': styles}).encode())
    marker = b'\n// EPSX_DEV_LIVE_CSS\n'
    before = stamp(bootstrap)
    if time.time_ns() - before[1] < 1_000_000_000:
        return  # Let DX finish writing the module before adding the loader.
    javascript = bootstrap.read_bytes()
    if marker not in javascript:
        # Generated dev bundle only; production source/assets are never patched.
        write_changed(bootstrap, javascript + marker + b'import("./epsx-dev-live-css.js").catch(console.error);\n', expected=before)


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
                try:
                    sync_live_css(REPO, bundle)
                except OSError:
                    pass  # DX may be replacing a bundle; retry after it finishes.
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
