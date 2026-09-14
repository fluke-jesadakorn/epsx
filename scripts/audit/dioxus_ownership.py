#!/usr/bin/env python3
"""Prevent a second DOM owner in migrated Fullstack code.

This checks active migration modules. dioxus_inventory.py --strict remains the
separate whole-repository retirement gate; passing here does not imply that all
legacy routes have been migrated.

Permitted adapters: wallet provider/chain calls, clipboard, localStorage display
preferences, focus/file APIs, PushManager/service worker, and EventSource streams.
They may return data to signals and release resources on unmount. They must not
create UI or install document/window click, change, submit or input delegates.
"""
from pathlib import Path
import re
from dioxus_retirement import production_source

ROOT = Path(__file__).resolve().parents[2]
UI = ROOT / 'shared/rust/dioxus_ui/src'
PATTERNS = {
    'raw HTML UI': r'dangerous_inner_html\s*:',
    'manual HTML replacement': r'(?:\.(?:set_inner_html|set_outer_html|replace_child|replaceChildren)\s*\(|\.(?:innerHTML|outerHTML)\s*=)',
    'global UI delegate': r'(?:document|window)\s*\.\s*addEventListener\s*\(\s*[\'"](?:click|change|submit|input|pointerdown|keydown)[\'"]',
    'legacy runtime bootstrap': r'epsx_browser_runtime_bootstrap|data-epsx-generated-runtime',
    'fragment rendering': r'dioxus_ssr::(?:render|render_element)\s*\(',
}


def main():
    paths = set((UI / 'fullstack').rglob('*.rs')) | set((UI / 'fullstack').rglob('*.js'))
    paths |= set(UI.rglob('hydrated.rs'))
    paths |= {UI / 'pages/account/push.rs'}
    findings = []
    for path in sorted(paths):
        source = path.read_text()
        # Unit-test SSR assertions are not UI runtime owners.
        source = production_source(source) if path.suffix == '.rs' else source
        # Dioxus 0.7 adds hydration markers to dynamic textarea raw text.
        # This exact adapter escapes text before Dioxus renders its own node;
        # it cannot accept HTML fragments or manipulate the DOM.
        if path == UI / 'fullstack/admin_textarea.rs':
            source = re.sub(r'dangerous_inner_html\s*:\s*escape_text\(&value\)', '', source)
        if path == UI / 'fullstack/pay/ui.rs':
            # Build-time repository documentation, never remote/page fragments.
            docs = source.split('fn Docs(merchant: bool)', 1)
            if len(docs) == 2 and 'include_str!' in docs[1]:
                source = docs[0] + 'fn Docs(merchant: bool)' + re.sub(r'dangerous_inner_html\s*:\s*html', '', docs[1])
        for label, pattern in PATTERNS.items():
            for match in re.finditer(pattern, source):
                line = source.count('\n', 0, match.start()) + 1
                findings.append(f'{path.relative_to(ROOT)}:{line}: {label}')
    if findings:
        raise SystemExit('\n'.join(findings))
    print(f'{len(paths)} migrated modules: Dioxus owns UI; no global UI delegates or HTML replacement.')


if __name__ == '__main__':
    main()
