"""Verify an immutable native release locally without real services or credentials."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import time
from urllib.error import HTTPError, URLError
from urllib.request import urlopen
from urllib.parse import urljoin


def request(url):
    try:
        response = urlopen(url, timeout=5)
    except HTTPError as error:
        response = error
    with response:
        return response.status, response.headers, response.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("release", type=Path)
    args = parser.parse_args()
    release = args.release.resolve()
    output = Path(__file__).resolve().parents[2] / "target/dioxus-migration/package"
    output.mkdir(parents=True, exist_ok=True)
    manifest = json.loads((release / "manifest.json").read_text())
    for item in manifest["files"]:
        relative = Path(item["path"])
        assert not relative.is_absolute() and ".." not in relative.parts
        path = release / relative
        assert not path.is_symlink() and path.is_file(), relative
        assert hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"], relative
    assert len(list((release / "bin").iterdir())) == 10
    runtime = release / "runtime"
    worker_digest = hashlib.sha256(
        (runtime / "epsx_service_worker.js").read_bytes()
        + (runtime / "epsx_service_worker_bg.wasm").read_bytes()
    ).hexdigest()
    bootstrap = (runtime / "epsx_service_worker_bootstrap.v3.js").read_text()
    assert f"epsx_service_worker.js?rev={worker_digest}" in bootstrap
    assert f"epsx_service_worker_bg.wasm?rev={worker_digest}" in bootstrap
    assert "/public/enterprise.css?v=dioxus-2" in bootstrap
    records = []
    for app, route in [("frontend", "/auth"), ("admin", "/auth"), ("pay", "/docs")]:
        public = release / "fullstack" / app / "public"
        wasm = list(public.rglob("*.wasm"))
        assert wasm and all(path.read_bytes()[:4] == b"\0asm" for path in wasm)
        assert os.access(release / "bin" / f"bff-{app}", os.X_OK)
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        origin = f"http://127.0.0.1:{port}"
        # No inherited signing keys, cookies, production URLs or dotenv files.
        env = dict(PATH=os.environ.get("PATH", ""), EPSX_ENV="development",
                   ENV="development", HOST="127.0.0.1", IP="127.0.0.1", PORT=str(port),
                   ROOT_ENV_FILE="/dev/null", RUST_LOG="error",
                   API_URL="http://127.0.0.1:1", BACKEND_URL="http://127.0.0.1:1",
                   PAYMENT_SERVICE_URL="http://127.0.0.1:1",
                   OIDC_ISSUER="http://127.0.0.1:1", OIDC_JWKS_URL="http://127.0.0.1:1/jwks",
                   FRONTEND_URL=origin, ADMIN_FRONTEND_URL=origin, PAY_FRONTEND_URL=origin,
                   DIOXUS_PUBLIC_PATH=str(public), EPSX_PUBLIC_DIR=str(release / "public" / app),
                   EPSX_BROWSER_RUNTIME_DIR=str(release / "runtime"))
        with (output / f"{app}.log").open("w") as log:
            process = subprocess.Popen([str(release / "bin" / f"bff-{app}")],
                                       cwd=release, env=env, stdout=log, stderr=log)
            try:
                for _ in range(100):
                    assert process.poll() is None, f"{app} exited; see {log.name}"
                    try:
                        if request(origin + "/api/health")[0] == 200:
                            break
                    except (URLError, TimeoutError):
                        pass
                    time.sleep(.1)
                else:
                    raise AssertionError(f"{app} never became healthy")
                status, headers, body = request(origin + route)
                html = body.decode()
                assert status == 200 and "text/html" in headers["content-type"]
                assert '<title' in html and 'id="main"' in html
                assert "epsx_browser_runtime_bootstrap" not in html
                scripts = re.findall(r'<script[^>]*src="([^"]+)"', html)
                modules = [src for src in scripts if "/wasm/" in src or "/assets/" in src]
                assert modules, f"{app} is SSR-only"
                for src in modules:
                    assert request(urljoin(origin + "/", src))[0] == 200
                for path in wasm:
                    assert request(origin + "/" + str(path.relative_to(public)))[2][:4] == b"\0asm"
                assert request(origin + "/not-a-real-dioxus-route")[0] == 404
                for other in {"frontend", "admin", "pay"} - {app}:
                    endpoint = "read" if other == "pay" else "auth_session"
                    assert request(origin + f"/_server/{other}/{endpoint}")[0] == 404
                if app == "frontend":
                    assert "/public/enterprise.css?v=dioxus-2" in html
                    css = request(origin + "/public/enterprise.css?v=dioxus-2")[2].decode()
                    assert "padding: clamp(20px, 3vw, 32px)" in css
                records.append({"app": app, "ssr": route, "wasm_files": len(wasm),
                                "asset_http": "passed", "unknown_route": 404,
                                "other_app_functions": "not mounted"})
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
    report = {"release": str(release), "manifest_files_verified": len(manifest["files"]),
              "native_binaries": 10, "worker_pair_sha256": worker_digest, "apps": records}
    (output / "report.json").write_text(json.dumps(report, indent=2))
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
