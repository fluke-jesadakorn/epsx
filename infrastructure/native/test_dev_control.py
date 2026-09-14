"""LaunchAgent orchestration tests; never touch real services or configuration."""
import importlib.util
import pathlib
import tempfile
import json
import os
import socket
import subprocess
import sys
import time
import unittest
from unittest.mock import patch, call
from io import BytesIO

spec = importlib.util.spec_from_file_location("control", pathlib.Path(__file__).with_name("dev-control.py"))
control = importlib.util.module_from_spec(spec)
spec.loader.exec_module(control)
asset_spec = importlib.util.spec_from_file_location("assets", pathlib.Path(__file__).with_name("dev-ui-assets.py"))
assets = importlib.util.module_from_spec(asset_spec)
asset_spec.loader.exec_module(assets)


class AssetTests(unittest.TestCase):
    def test_live_css_updates_generated_bundle_without_changing_source_or_duplicating_loader(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            native = root/'infrastructure/native'
            native.mkdir(parents=True)
            (native/'dev-live-css.js').write_text('// dev runtime')
            public = root/'apps/frontend/public'
            public.mkdir(parents=True)
            (public/'dist').mkdir()
            source = public/'dist/tailwind.css'
            source.write_text('body { color: red; }')
            bundle = root/'target/dx/dx-frontend/debug/web/public/wasm'
            bundle.mkdir(parents=True)
            bootstrap = bundle/'dx-frontend.js'
            bootstrap.write_text('// generated DX module')
            os.utime(bootstrap, (time.time() - 2, time.time() - 2))
            assets.sync_live_css(root, 'dx-frontend')
            manifest = json.loads((bundle/'epsx-dev-live-css.json').read_text())
            asset = manifest['styles']['/public/dist/tailwind.css']
            self.assertEqual((bundle/asset['file']).read_text(), source.read_text())
            source.write_text('body { color: blue; }')
            assets.sync_live_css(root, 'dx-frontend')
            updated = json.loads((bundle/'epsx-dev-live-css.json').read_text())
            self.assertNotEqual(manifest['revision'], updated['revision'])
            self.assertEqual((bundle/asset['file']).read_text(), source.read_text())
            self.assertEqual(bootstrap.read_text().count('EPSX_DEV_LIVE_CSS'), 1)

    def test_live_loader_never_overwrites_a_newer_dx_bundle(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory)/'app.js'
            path.write_bytes(b'old build')
            before = assets.stamp(path)
            path.write_bytes(b'new build')
            assets.write_changed(path, b'old build with loader', expected=before)
            self.assertEqual(path.read_bytes(), b'new build')

    def test_strip_preserves_non_debug_sections_and_is_idempotent(self):
        header = b"\0asm\x01\0\0\0"
        keep = b"\x00\x06\x04name\x00"
        debug = b"\x00\x0c\x0b.debug_info"
        self.assertEqual(assets.strip_dwarf(header + keep + debug), header + keep)
        self.assertEqual(assets.strip_dwarf(header + keep), header + keep)
        with self.assertRaises(ValueError): assets.strip_dwarf(header + debug[:-1])

    def test_partial_bundle_is_never_replaced(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "app.wasm"
            original = b"\0asm\x01\0\0\0\x00\x7f"
            path.write_bytes(original)
            with self.assertRaises(ValueError): assets.compact(path)
            self.assertEqual(path.read_bytes(), original)


class RealtimeTests(unittest.TestCase):
    def test_disables_automatic_rebuilds_and_requires_explicit_rebuild(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            address = root/'control.sock'
            fake = root/'dx.py'
            fake.write_text('''import os, tty
tty.setraw(0)
print("Serving your app: test", flush=True)
while True:
    key = os.read(0, 1)
    if key == b'p': print("Automatic rebuilds are currently: disabled", flush=True)
    elif key == b'r': print("MANUAL_REBUILD", flush=True)
''')
            wrapper = pathlib.Path(__file__).with_name('dev-ui-realtime.py')
            code = '''import importlib.util, pathlib, sys
spec = importlib.util.spec_from_file_location('realtime', sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
module.control_path = lambda _: pathlib.Path(sys.argv[2])
module.serve('bff-frontend', [sys.executable, sys.argv[3]])
'''
            log = root/'output'
            with log.open('w') as output:
                process = subprocess.Popen([sys.executable, '-c', code, str(wrapper), str(address), str(fake)],
                                           stdout=output, stderr=output)
                try:
                    def request(action):
                        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as client:
                            client.settimeout(1)
                            client.connect(str(address))
                            client.sendall(action.encode())
                            return json.loads(client.recv(4096))
                    deadline = time.monotonic() + 8
                    result = None
                    while time.monotonic() < deadline:
                        try:
                            result = request('status')
                            if result['automatic_rebuilds'] is False: break
                        except OSError: pass
                        time.sleep(0.05)
                    self.assertIsNotNone(result, log.read_text())
                    self.assertFalse(result['automatic_rebuilds'], log.read_text())
                    self.assertNotIn('MANUAL_REBUILD', log.read_text())
                    self.assertIn('error', request('toggle'))
                    self.assertTrue(request('rebuild')['rebuild_requested'])
                    while time.monotonic() < deadline and 'MANUAL_REBUILD' not in log.read_text():
                        time.sleep(0.05)
                    self.assertIn('MANUAL_REBUILD', log.read_text())
                finally:
                    process.terminate()
                    process.wait(timeout=12)
                self.assertFalse(address.exists())


class HmrTests(unittest.TestCase):
    def test_realtime_alias_starts_hmr_and_never_restores_old_processes(self):
        with patch.object(sys, 'argv', ['dev-control.py', 'realtime', 'bff-frontend']), \
             patch.object(control, 'hmr') as hmr, patch.object(control, 'restore_hmr') as restore:
            control.main()
            hmr.assert_called_once_with(['bff-frontend'])
            restore.assert_not_called()

    def test_loading_page_is_not_ready(self):
        loading = BytesIO(b'<html>Building...</html>')
        loading.status = 200
        ready = BytesIO(b'<html><div id="main">App</div></html>')
        ready.status = 200
        with patch.object(control.urllib.request, "urlopen", side_effect=[loading, ready]) as fetch, \
             patch.object(control.time, "sleep") as sleep:
            control.wait_ui("bff-frontend", timeout=10)
            self.assertEqual(fetch.call_count, 2)
            sleep.assert_called_once_with(2)

    def test_ui_only_startup_waits_between_apps(self):
        with tempfile.TemporaryDirectory() as directory:
            events = []
            with patch.object(control, "ROOT", pathlib.Path(directory)), \
                 patch.object(control, "CHECKOUT", pathlib.Path(directory)), \
                 patch.object(control.subprocess, "run") as run, \
                 patch.object(control, "enable_hmr", side_effect=lambda name: events.append(("start", name))), \
                 patch.object(control, "wait_ui", side_effect=lambda name: events.append(("ready", name))):
                control.hmr(control.UI)
                self.assertEqual(events, [("start", "ui-worker"),
                    ("start", "bff-frontend"), ("ready", "bff-frontend"),
                    ("start", "bff-admin"), ("ready", "bff-admin"),
                    ("start", "bff-pay"), ("ready", "bff-pay")])
                env = run.call_args.kwargs["env"]
                self.assertEqual(env["CARGO_TARGET_DIR"], str(control.CHECKOUT / "target"))
                self.assertEqual(env["CARGO_BUILD_JOBS"], "2")

    def test_existing_runtime_is_reused_without_cargo(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            runtime = root/'target/epsx-service-worker/epsx_service_worker_bootstrap.js'
            runtime.parent.mkdir(parents=True)
            runtime.write_text('existing runtime')
            with patch.object(control, "ROOT", root), patch.object(control, "CHECKOUT", root), \
                 patch.object(control.subprocess, "run") as run, \
                 patch.object(control, "enable_hmr"), patch.object(control, "wait_ui"):
                control.hmr(['bff-frontend'])
                run.assert_not_called()

    def test_enable_is_idempotent_and_restore_preserves_configuration(self):
        with tempfile.TemporaryDirectory() as directory:
            paths = lambda name: pathlib.Path(directory) / (name + ".plist")
            path = paths("bff-frontend")
            original = control.plistlib.dumps({"Label": "test", "ProgramArguments": ["original"],
                                              "EnvironmentVariables": {"EXISTING": "preserved"}})
            path.write_bytes(original)
            with patch.object(control, "path", side_effect=paths), \
                 patch.object(control, "loaded", return_value=True), patch.object(control, "run") as run:
                control.enable_hmr("bff-frontend")
                self.assertEqual(path.with_suffix(".plist.pre-hmr").read_bytes(), original)
                variables = control.plistlib.loads(path.read_bytes())["EnvironmentVariables"]
                self.assertEqual(variables["EXISTING"], "preserved")
                self.assertEqual(variables["EPSX_DEV_UI_MODE"], "realtime-v1")
                run.reset_mock()
                control.enable_hmr("bff-frontend")
                run.assert_not_called()
                control.restore_hmr(["bff-frontend"])
                self.assertEqual(path.read_bytes(), original)
                self.assertNotIn(call("launchctl", "bootout", control.DOMAIN, str(paths("epsx"))), run.call_args_list)

    def test_missing_backup_fails_before_stopping_any_service(self):
        with tempfile.TemporaryDirectory() as directory, \
             patch.object(control, "path", side_effect=lambda name: pathlib.Path(directory) / name), \
             patch.object(control, "run") as run:
            with self.assertRaises(SystemExit): control.restore_hmr(control.UI)
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
