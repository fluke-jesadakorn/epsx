#!/usr/bin/env python3
"""Inventory UI migration evidence; reports do not equate SSR markup with hydration.

Outputs a reproducible JSON report under target/. --strict rejects legacy UI
owners reachable from deployment entrypoints. Retired findings stay in the report
with explicit test-item or unlinked-crate evidence.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
from dioxus_retirement import test_ranges, test_module_files, boundary_errors

ROOT = Path(__file__).resolve().parents[2]
SOURCES = ('apps/frontend/src', 'apps/admin/src', 'apps/pay/src',
           'shared/rust/dioxus_ui/src', 'shared/rust/browser-runtime/src',
           'shared/rust/templates/src')
PATTERNS = {
    'axum_route': r'\.route\(\s*"([^"\n]+)"',
    'dioxus_route': r'#\[route\("([^"\n]+)"',
    'server_function': r'#\[server(?:\([^\]]*\))?\]',
    'hydrated_loader': r'\buse_server_future\s*\(',
    'legacy_ssr_render': r'\bdioxus_ssr::(?:render|render_element)\s*\(',
    'legacy_dom_write': r'\.(?:set_inner_html|set_outer_html|replace_child)\s*\(',
    'legacy_event_binding': r'\bfn\s+(bind_[a-z_]+)\s*\(',
    'html_document': r'<!DOCTYPE html|<!doctype html',
}


def inventory():
    findings = []
    test_files = set().union(*(test_module_files(ROOT / directory) for directory in SOURCES))
    for directory in SOURCES:
        for path in sorted((ROOT / directory).rglob('*.rs')):
            source = path.read_text()
            ranges = test_ranges(source)
            for kind, pattern in PATTERNS.items():
                for match in re.finditer(pattern, source):
                    findings.append({'kind': kind, 'file': str(path.relative_to(ROOT)),
                                     'line': source.count('\n', 0, match.start()) + 1,
                                     'value': match.group(1) if match.lastindex else match.group(0),
                                     'ownership': 'test-only' if path in test_files or any(a <= match.start() < b for a,b in ranges) else ('retired-runtime-candidate' if str(path.relative_to(ROOT)) == 'shared/rust/browser-runtime/src/lib.rs' else 'production')})
    return findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--strict', action='store_true')
    parser.add_argument('--output', type=Path, default=ROOT / 'target/dioxus-migration/inventory.json')
    args = parser.parse_args()
    findings = inventory()
    errors = boundary_errors(ROOT)
    unlinked = True
    if args.strict:
        host = next(line.split(': ',1)[1] for line in subprocess.check_output(['rustc','-vV'],text=True).splitlines() if line.startswith('host: '))
        for package in ('epsx-frontend','epsx-admin','epsx-pay-bff'):
            for feature,target in [('server',host),('web','wasm32-unknown-unknown')]:
                graph = subprocess.check_output(['cargo','tree','--locked','-p',package,'--no-default-features','--features',feature,'--target',target,'--prefix','none','--format','{p}'],text=True,stderr=subprocess.DEVNULL)
                if re.search(r'^epsx-browser-runtime ',graph,re.M):
                    unlinked = False
                    errors.append(f'{package}/{feature} links the retired browser runtime')
        for finding in findings:
            if finding['ownership']=='retired-runtime-candidate' and unlinked and not errors:
                finding['ownership']='retired-unlinked-crate'
    counts = {kind: sum(f['kind'] == kind for f in findings) for kind in PATTERNS}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({'counts': counts, 'boundary_errors': errors, 'findings': findings}, indent=2) + '\n')
    print(json.dumps(counts, indent=2))
    print(f'Report: {args.output}')
    legacy = [f for f in findings if f['kind'].startswith('legacy_') and f['ownership'] not in ('test-only','retired-unlinked-crate')]
    if args.strict and (legacy or errors):
        raise SystemExit('\n'.join(errors+[f"{f['file']}:{f['line']}: active {f['kind']}" for f in legacy]))
    if args.strict: print('All native entrypoints and six deployment graphs exclude retired UI owners; test references remain audited.')


if __name__ == '__main__':
    main()
