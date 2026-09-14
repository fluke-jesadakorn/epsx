import unittest
from pathlib import Path
import tempfile
import shutil
from dioxus_retirement import production_source, boundary_errors
ROOT=Path(__file__).resolve().parents[2]
class RetirementTests(unittest.TestCase):
    def test_test_item_does_not_hide_later_production_code(self):
        source='#[cfg(test)] mod tests { fn x(){ let s="}"; } }\nfn live(){dioxus_ssr::render(x);}'
        result=production_source(source)
        self.assertNotIn('fn x',result)
        self.assertIn('dioxus_ssr::render',result)
    def test_test_statement_preserves_following_native_statement(self):
        source='#[cfg(test)] let runtime = if old { browser_runtime_router() } else { worker };\nlaunch();'
        result=production_source(source)
        self.assertNotIn('browser_runtime_router',result)
        self.assertIn('launch()',result)
    def test_current_native_boundaries(self):
        self.assertEqual(boundary_errors(ROOT),[])
    def test_reintroduced_native_owner_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            files=[f'apps/{a}/src/{f}.rs' for a in ('frontend','admin','pay') for f in ('lib','fullstack','dx_main')]+['shared/rust/dioxus_ui/src/routes.rs']
            for name in files:
                p=root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy(ROOT/name,p)
            file=root/'apps/admin/src/lib.rs';file.write_text(file.read_text()+'\nfn regression(){ browser_runtime_router("x"); }')
            self.assertTrue(any('browser_runtime_router' in e for e in boundary_errors(root)))
if __name__=='__main__':unittest.main()
