"""Real SSR + WASM hydration audit against an isolated backend fixture.
Run dx build --fullstack true -p epsx-frontend --bin dx-frontend --web first.
No user account, production routes, or database are used.
"""
import json
import os
from pathlib import Path
import socket
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse
from urllib.request import urlopen
from urllib.error import HTTPError

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/dioxus-migration/analytics'
BUNDLE = Path(os.environ.get('EPSX_AUDIT_FRONTEND_BUNDLE', str(ROOT / 'target/dx/dx-frontend/debug/web')))
MODE = 'ready'
ACCESS_NOTE = False
READS = []


class Backend(BaseHTTPRequestHandler):
    def log_message(self, *_): pass

    def do_GET(self):
        global READS
        path = urlparse(self.path).path
        if path.endswith('/filters'):
            body = {'countries': [{'value': 'america', 'label': 'United States'}], 'sectors': ['Technology'], 'exchanges': [], 'stock_types': []}
        elif path.endswith('/rankings'):
            query = parse_qs(urlparse(self.path).query)
            READS.append(query)
            limit = int(query.get('limit', ['10'])[0])
            page = int(query.get('page', ['1'])[0])
            if MODE == 'error':
                self.send_error(503)
                return
            if MODE == 'race' and limit == 25: time.sleep(.5)
            count = 0 if MODE == 'empty' else limit
            rows = [{'rank': (page - 1) * limit + n + (2 if ACCESS_NOTE else 1), 'symbol': f'TEST{n}', 'company_name': f'Test Company {n}',
                     'latest_date': '2026-09-09', 'value': 90., 'active_status': 'TRACK',
                     'quarterly_performance': [], 'next_earnings_date': 1792454400,
                     'last_earnings_date': 1784505600} for n in range(count)]
            body = {'success': True, 'data': rows, 'pagination': {'page': page, 'limit': limit, 'total': 200 if count else 0,
                    'totalPages': (200 + limit - 1) // limit if count else 0, 'hasNext': page * limit < 200 and count > 0, 'hasPrev': page > 1},
                    'metadata': {'available_countries': ['america'], 'available_sectors': ['Technology'],
                    'request_timestamp': '2026-09-09T00:00:00Z', 'data_source': 'isolated-fullstack-audit'},
                    'access_info': {'min_accessible_rank':2,'locked_ranks_count':1,'max_accessible_rank':None} if ACCESS_NOTE else None, 'message': None, 'processing_time_ms': 1}
        else:
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())


def browser(*args):
    result = subprocess.run(['agent-browser', '--session', 'epsx-dioxus-audit', *args],
                            capture_output=True, text=True, timeout=35)
    if result.returncode:
        raise RuntimeError(f'Browser {args}: {result.stderr} {result.stdout}')
    return result.stdout.strip()


def main():
    global MODE, ACCESS_NOTE
    OUT.mkdir(parents=True, exist_ok=True)
    upstream = ThreadingHTTPServer(('127.0.0.1', 0), Backend)
    threading.Thread(target=upstream.serve_forever, daemon=True).start()
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    base = f'http://127.0.0.1:{port}'
    backend = f'http://127.0.0.1:{upstream.server_port}'
    env = dict(os.environ, EPSX_ENV='development', ENV='development', IP='127.0.0.1', HOST='127.0.0.1', PORT=str(port),
               API_URL=backend, BACKEND_URL=backend, CONTENT_SERVICE_URL=backend, NOTIFICATION_SERVICE_URL=backend,
               OIDC_ISSUER=backend, OIDC_JWKS_URL=f'{backend}/.well-known/jwks.json', DIOXUS_PUBLIC_PATH=str(BUNDLE / 'public'))
    records = []
    with (OUT / 'server.log').open('w') as log:
        process = subprocess.Popen([str(BUNDLE / 'server')], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if process.poll() is not None: raise RuntimeError('Fullstack server exited; see server.log')
                try:
                    with urlopen(base + '/api/health', timeout=.3): break
                except OSError: time.sleep(.1)
            else: raise RuntimeError('Fullstack server did not start')
            query = '?page=1&limit=10&country=america&sector=Technology&sort_by=growth_factor'
            browser('open', base + '/analytics' + query)
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('wait', '.fe-ranking-card')
            assert len(READS) == 1, ('hydration repeated the SSR read', READS)
            browser('eval', 'window.originalDocument=document;window.originalShell=document.querySelector(".fe-sidebar");')
            for limit in [25, 50, 100, 10]:
                browser('select', '#analytics-limit', str(limit))
                browser('wait', '--fn', f'document.querySelectorAll(".fe-ranking-card").length === {limit} && document.querySelector("[data-dioxus-analytics]").getAttribute("aria-busy") === "false"')
                actual = parse_qs(urlparse(browser('get', 'url')).query)
                assert actual['limit'] == [str(limit)] and actual['country'] == ['america'] and actual['page'] == ['1'], actual
                assert browser('eval', 'document === originalDocument && document.querySelector(".fe-sidebar") === originalShell') == 'true'
                records.append({'limit': limit, 'same_document': True})
            browser('back')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 100')
            browser('forward')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 10')
            browser('back')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 100')
            records.append({'history_back_and_forward': True})
            MODE = 'error'
            browser('select', '#analytics-limit', '25')
            browser('wait', '--text', 'Could not load results')
            assert browser('eval', 'document.querySelectorAll(".fe-ranking-card").length') == '100'
            assert browser('get', 'value', '#analytics-limit') == '100'
            MODE = 'ready'
            browser('click', '[data-dioxus-analytics] > div[role="status"] > button')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 25 && document.querySelector("[data-dioxus-analytics]").getAttribute("aria-busy") === "false"')
            records.append({'failure_keeps_previous_data_and_retry': True})
            browser('select', '#analytics-limit', '10')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 10')
            race_start = len(READS)
            MODE = 'race'
            browser('select', '#analytics-limit', '25')
            browser('select', '#analytics-limit', '50')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 50')
            time.sleep(.7)
            assert browser('eval', 'document.querySelectorAll(".fe-ranking-card").length') == '50'
            assert any(read.get('limit') == ['25'] for read in READS[race_start:]), READS[race_start:]
            assert any(read.get('limit') == ['50'] for read in READS[race_start:]), READS[race_start:]
            records.append({'latest_request_wins': True})
            MODE = 'empty'
            browser('select', '#analytics-limit', '10')
            browser('wait', '--text', 'No companies match these filters')
            records.append({'empty': True})
            MODE = 'ready'
            browser('back')
            browser('wait', '--fn', 'document.querySelectorAll(".fe-ranking-card").length === 50')
            browser('set', 'viewport', '1440', '900')
            browser('screenshot', str(OUT / 'desktop.png'))
            browser('set', 'viewport', '390', '844')
            browser('screenshot', str(OUT / 'mobile.png'))
            assert browser('eval', 'document.documentElement.scrollWidth <= innerWidth') == 'true'
            browser('set', 'viewport', '1440', '900')
            browser('scrollintoview', '.fe-ranking-order a')
            browser('click', '.fe-ranking-order a')
            browser('wait', '--text', 'Order: EPSX ranking')
            assert browser('eval', 'document === originalDocument') == 'true'
            ACCESS_NOTE = True
            browser('select', '#analytics-limit', '10')
            browser('wait', '.fe-access-note a[href="/plans"]')
            browser('scrollintoview', '.fe-access-note a[href="/plans"]')
            browser('click', '.fe-access-note a[href="/plans"]')
            browser('wait', '--fn', 'location.pathname === "/plans"')
            assert browser('eval', 'document === originalDocument') == 'true'
            browser('back')
            browser('wait', '[data-dioxus-analytics]')
            assert browser('eval', 'document === originalDocument') == 'true'
            records.append({'ranking_reset_and_review_plans_links_no_reload': True})
            browser('click', '.fe-brand')
            browser('wait', '[data-home-market-state="ready"]')
            assert browser('eval', 'document === originalDocument') == 'true'
            assert browser('eval', 'document.querySelectorAll(".fe-ranking-card").length') == '2'
            browser('screenshot', str(OUT / 'home.png'))
            browser('click', '.fe-footer a[href="/about"]')
            browser('wait', '--fn', 'location.pathname === "/about"')
            assert browser('eval', 'document === originalDocument && !!document.querySelector(".fe-marketing-header")') == 'true'
            records.append({'home_and_marketing_spa': True})
            for theme in ['light', 'dark']:
                current = browser('eval', 'document.querySelector(".epsx-frontend").getAttribute("data-theme")').strip('"')
                if current != theme:
                    browser('click', 'button[aria-label="Toggle theme"]')
                browser('wait', '--fn', f'localStorage.getItem("epsx-theme") === "{theme}" && document.querySelector(".epsx-frontend").getAttribute("data-theme") === "{theme}"')
                browser('scrollintoview', '.fe-footer a[href="/privacy"]')
                browser('click', '.fe-footer a[href="/privacy"]')
                browser('wait', '--fn', 'location.pathname === "/privacy"')
                assert browser('eval', f'document === originalDocument && document.querySelector(".epsx-frontend").getAttribute("data-theme") === "{theme}"') == 'true'
                browser('back')
                browser('wait', '--fn', 'location.pathname === "/about"')
                assert browser('eval', f'document === originalDocument && document.querySelector(".epsx-frontend").getAttribute("data-theme") === "{theme}" && document.querySelector(".epsx-frontend").classList.contains("dark") === {str(theme=="dark").lower()}') == 'true'
                browser('screenshot', str(OUT / f'theme-{theme}.png'))
            records.append({'light_dark_theme_persists_navigation_and_back': True})
            for path, expected in [('/analytics?limit=0', 400), ('/news/missing-article', 404)]:
                try:
                    with urlopen(base + path) as response:
                        actual_status = response.status
                except HTTPError as error:
                    actual_status = error.code
                assert actual_status == expected, (path, actual_status, expected)
            records.append({'ssr_error_statuses': True})
            (OUT / 'results.json').write_text(json.dumps(records, indent=2))
            print(json.dumps(records, indent=2))
        except Exception:
            (OUT / 'failure-snapshot.txt').write_text(browser('snapshot'))
            (OUT / 'failure-errors.txt').write_text(browser('errors'))
            browser('screenshot', str(OUT / 'failure.png'))
            raise
        finally:
            browser('close')
            process.terminate()
            process.wait(timeout=10)
            upstream.shutdown()
            upstream.server_close()


if __name__ == '__main__': main()
