"""Capture a separate enterprise baseline against loopback-only native BFFs.

Requires the installed agent-browser CLI; no app dependency or browser bundle
is added to EPSX. Run the Rust fixture, fixture_adapter.py, frontend :3300 and
session_proxy.py first. Every result and screenshot goes under target/.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/enterprise-redesign/after'
SESSION = os.environ.get('EPSX_QA_BROWSER_SESSION', 'epsx-enterprise-captures')
ROUTES = [
    '/', '/analytics', '/portfolio', '/dashboard', '/account', '/profile',
    '/permissions', '/account/credits', '/payment', '/plans', '/notifications',
    '/chat', '/chat/history', '/developer', '/developer/docs', '/developer/usage',
    '/news', '/manual', '/about', '/contact', '/terms', '/privacy', '/offline',
    '/access-denied', '/missing-page', '/auth?return_url=%2Fanalytics',
    '/pricing?ref=enterprise-qa',
    '/portfolio/0xea6400000000000000000000000000000000e3df',
    '/chat/550e8400-e29b-41d4-a716-446655440000',
    '/news/deterministic-market-brief',
    '/payment/plan/00000000-0000-0000-0000-000000000001',
]
MEASURE = '''(() => ({
  url: location.pathname + location.search,
  width: innerWidth,
  dark: document.documentElement.classList.contains('dark'),
  overflow: document.documentElement.scrollWidth > innerWidth + 1,
  mains: document.querySelectorAll('main').length,
  headings: Array.from(document.querySelectorAll('h1')).map(e => e.textContent),
  styles: !!document.querySelector('link[href^="/public/enterprise.css"]'),
  active: Array.from(document.querySelectorAll('a[aria-current="page"]')).map(e => e.getAttribute('href')),
  states: Array.from(document.querySelectorAll('[data-state], [data-analytics-state], [data-watchlist-state]')).map(e => Array.from(e.attributes).filter(a => a.name.endsWith('state')).map(a => [a.name,a.value]))
}))()'''


def browser(*args, data=False):
    result = subprocess.run(['agent-browser', '--session', SESSION, '--json', *args],
                            capture_output=True, text=True, timeout=45)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    value = json.loads(result.stdout)
    if not value.get('success'):
        raise RuntimeError(value)
    return value.get('data', {}) if data else None


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--widths', nargs='+', type=int, default=[375,768,1024,1440])
    args = parser.parse_args()
    OUT.mkdir(parents=True, exist_ok=True)
    records = []
    browser('--args', '--disable-gpu', 'open', 'http://127.0.0.1:3300/')
    for width in args.widths:
        browser('set', 'viewport', str(width), '900')
        for theme in ('light','dark'):
            browser('set', 'media', theme, 'reduced-motion')
            for route in ROUTES:
                # Auth uses the signed-out surface. Other pages use only the
                # fixture token in the loopback proxy, never a real account.
                origin = 'http://127.0.0.1:3300' if route.startswith('/auth') else 'http://127.0.0.1:3301'
                browser('open', origin + route)
                browser('wait', '--fn', "getComputedStyle(document.body).getPropertyValue('--fe-bg').trim() !== '' && document.readyState === 'complete'")
                value = browser('eval', MEASURE, data=True)['result']
                name = ('home' if route == '/' else route.strip('/').replace('/','-').split('?')[0])
                image = f'{name}-{width}-{theme}.png'
                # Viewport captures avoid a Chromium full-page compositor
                # timeout on this Mac. Overflow is measured for the whole DOM.
                browser('screenshot', str(OUT / image))
                value.update(route=route, theme=theme, screenshot=image,
                             ok=not value['overflow'] and value['mains']==1 and len(value['headings'])==1
                                and value['styles'] and value['dark']==(theme=='dark'))
                records.append(value)
                (OUT / 'audit.json').write_text(json.dumps(records, indent=2))
            print(f'{width}px {theme}: {len(ROUTES)} routes captured', flush=True)
    issues = [r for r in records if not r['ok']]
    print(json.dumps({'checks':len(records), 'issues':issues}, indent=2), flush=True)
    return bool(issues)


if __name__ == '__main__':
    raise SystemExit(main())
