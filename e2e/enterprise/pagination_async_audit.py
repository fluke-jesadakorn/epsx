"""Exercise generated WASM page-size updates against an isolated SSR fixture.

Build epsx-browser-runtime for wasm32 and run wasm-bindgen --target web with
--out-dir target/pagination-ui before running this audit. Requires agent-browser.
"""
import json
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import subprocess
from threading import Thread
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / 'target/pagination-ui'
MODE = 'ready'


class Handler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ASSETS), **kwargs)

    def log_message(self, *_):
        pass

    def do_GET(self):
        if urlparse(self.path).path != '/analytics':
            return super().do_GET()
        if MODE == 'error':
            self.send_error(503)
            return
        query = parse_qs(urlparse(self.path).query)
        limit = int(query.get('limit', ['10'])[0])
        options = ''.join(f'<option value="{n}" {"selected" if n == limit else ""}>{n}</option>' for n in [10, 25, 50, 100])
        rows = ''.join('<article data-company>Company</article>' for _ in range(limit)) if MODE == 'ready' else 'No companies match these filters'
        page = f'''<!doctype html><html><body><header id="shell">Workspace</header>
        <section data-section="analytics-rankings" data-analytics-state="{MODE}">
        <div data-count="{limit if MODE == 'ready' else 0}">{rows}</div>
        <form action="/analytics" method="get">
        <input type="hidden" name="page" value="1">
        <input type="hidden" name="country" value="america">
        <input type="hidden" name="sector" value="Technology">
        <input type="hidden" name="sort_by" value="growth_factor">
        <input type="hidden" name="min_eps" value="1.5">
        <input type="hidden" name="min_growth" value="-2">
        <select id="analytics-limit" name="limit" data-analytics-limit="true">{options}</select>
        </form></section><script type="module">
        import init from '/epsx_browser_runtime.js'; await init(); document.body.dataset.ready='true';
        </script></body></html>'''
        self.send_response(200)
        self.send_header('Content-Type', 'text/html')
        self.end_headers()
        self.wfile.write(page.encode())


def browser(*args):
    result = subprocess.run(['agent-browser', '--session', 'epsx-pagination-async', *args],
                            capture_output=True, text=True, check=True, timeout=30)
    return result.stdout.strip()


def main():
    global MODE
    server = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    Thread(target=server.serve_forever, daemon=True).start()
    records = []
    try:
        browser('open', f'http://127.0.0.1:{server.server_port}/analytics?page=4&limit=10')
        browser('wait', 'body[data-ready=true]')
        browser('eval', 'window.originalDocument=document;window.originalShell=document.querySelector("#shell");')
        for limit in [25, 50, 100, 10]:
            browser('select', '#analytics-limit', str(limit))
            browser('wait', f'[data-count="{limit}"]')
            query = parse_qs(urlparse(browser('get', 'url')).query)
            assert query == {'page': ['1'], 'limit': [str(limit)], 'country': ['america'],
                             'sector': ['Technology'], 'sort_by': ['growth_factor'],
                             'min_eps': ['1.5'], 'min_growth': ['-2']}, query
            assert browser('eval', 'document===window.originalDocument && document.querySelector("#shell")===window.originalShell') == 'true'
            records.append({'limit': limit, 'no_reload': True, 'filters_preserved': True})
        MODE = 'error'
        browser('select', '#analytics-limit', '25')
        browser('wait', '--text', 'Could not update results')
        assert browser('eval', 'document.querySelector("select").value === "10" && !document.querySelector("select").disabled && document.querySelectorAll("[data-company]").length === 10') == 'true'
        records.append({'failed_request_preserves_rows_and_selection': True})
        MODE = 'empty'
        browser('select', '#analytics-limit', '25')
        browser('wait', '[data-count="0"]')
        assert browser('eval', 'document===window.originalDocument') == 'true'
        records.append({'retry_and_empty_results_without_reload': True})
        (ASSETS / 'async-results.json').write_text(json.dumps(records, indent=2))
        print(json.dumps(records, indent=2))
    finally:
        browser('close')
        server.shutdown()
        server.server_close()


if __name__ == '__main__':
    main()
