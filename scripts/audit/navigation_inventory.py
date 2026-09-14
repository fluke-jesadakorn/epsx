#!/usr/bin/env python3
"""Inventory authored navigation and route declarations; no application writes.
Run from any directory. Output is evidence, not a claim of browser coverage.
"""
import json
import re
import sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/navigation-audit'
PATTERNS = {
    'shared_link': r'\b(?:AppLink|ShellLink)\s*\{',
    'dioxus_link': r'(?<!:)\bLink\s*\{',
    'native_anchor': r'\ba\s*\{',
    'form': r'\b(?:form|QueryForm)\s*\{',
    'router_navigation': r'\b(?:navigator|navigation|router)\.(?:push|replace)',
    'document_navigation': r'(?:window\.)?location\.(?:assign|replace|reload)|location\.href\s*=|\.location\(\)\.(?:assign|replace|reload|set_href)|\breload\(\)',
    'http_redirect': r'Redirect::|StatusCode::(?:FOUND|SEE_OTHER|TEMPORARY_REDIRECT|PERMANENT_REDIRECT)',
    'suspense': r'\b(?:SuspenseBoundary|use_server_future)\b',
}
# Skip comments and literals for RSX tokens; retain originals for JS/redirect evidence.
LITERALS = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"')
def main():
    sites, routes = [], []
    roots = [ROOT / 'apps' / app / 'src' for app in ('frontend', 'admin', 'pay')]
    roots += [ROOT / 'shared/rust/dioxus_ui/src', ROOT / 'shared/rust/browser-runtime/src', ROOT / 'shared/rust/service-worker/src']
    for folder in roots:
        for file in sorted(folder.rglob('*')):
            if file.suffix not in ('.rs', '.js'): continue
            text = file.read_text()
            if len(text) > 300_000 or 'walletconnect-2.' in file.name: continue
            relative = str(file.relative_to(ROOT))
            for number, line in enumerate(text.splitlines(), 1):
                for category, pattern in PATTERNS.items():
                    candidate = line if category in ('document_navigation', 'http_redirect') else LITERALS.sub('', line)
                    if re.search(pattern, candidate) and not (category == 'native_anchor' and 'match a {' in candidate):
                        sites.append({'file': relative, 'line': number, 'kind': category, 'code': line.strip()})
                match = re.search(r'#\[(?:route|redirect)\("([^"\n]+)"', line)
                if match: routes.append({'file': relative, 'line': number, 'route': match[1]})
    OUT.mkdir(parents=True, exist_ok=True)
    for site in sites:
        file, kind, code = site['file'], site['kind'], site['code']
        site['runtime_scope'] = 'legacy browser runtime (not loaded by fullstack roots)' if '/browser-runtime/' in file else 'authored application/SSR candidate; may include tests'
        if kind == 'document_navigation':
            site['review'] = ('legacy fallback' if '/browser-runtime/' in file else 'explicit reload/offline recovery' if file.endswith(('error_page.rs', 'offline.rs')) else 'external payment handoff' if file.endswith('wallet_adapter.js') else 'server/runtime candidate; inspect code context')
        elif kind == 'http_redirect':
            site['review'] = 'HTTP/SSR redirect; preserved for direct requests and native fallbacks'
        elif kind in ('shared_link','dioxus_link','native_anchor'):
            site['review'] = 'Router internal destination; adapter preserves native external/fragment/download/target behavior'
        elif kind == 'form':
            site['review'] = 'typed hydrated submit or standalone SSR fallback; see report for legacy renderers'
    result = {'routes': routes, 'sites': sites, 'counts': {kind: sum(x['kind'] == kind for x in sites) for kind in PATTERNS},
              'note': 'Static candidates, including standalone SSR and tests; browser coverage is reported separately.'}
    (OUT / 'inventory.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'routes': len(routes), **result['counts']}))
    if '--check' in sys.argv:
        unexpected = [site for site in sites if site['kind'] == 'native_anchor' and site['file'] != 'shared/rust/dioxus_ui/src/navigation.rs']
        allowed = {'shared/rust/dioxus_ui/src/fullstack/pay/wallet_adapter.js', 'shared/rust/dioxus_ui/src/pages/error_page.rs', 'shared/rust/dioxus_ui/src/pages/offline.rs'}
        unexpected += [site for site in sites if site['kind'] == 'document_navigation' and '/dioxus_ui/' in site['file'] and site['file'] not in allowed]
        if unexpected: raise SystemExit(json.dumps({'unreviewed_document_navigation': unexpected}, indent=2))
if __name__ == '__main__': main()
