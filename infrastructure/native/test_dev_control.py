"""LaunchAgent orchestration tests; never touch real services or configuration."""
import importlib.util
import pathlib
import tempfile
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


class HmrTests(unittest.TestCase):
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
                self.assertEqual(control.plistlib.loads(path.read_bytes())["EnvironmentVariables"],
                                 {"EXISTING": "preserved"})
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
