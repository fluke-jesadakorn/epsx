"""SSR/WASM regression: expired sessions, SIWE return URLs and pagination.

Uses isolated HTTP fixtures, an ephemeral signing key and a fake wallet. No
account, database, extension, or production service is touched.
"""
import json
import os
import shutil
import socket
import subprocess
import threading
import time
from http.server import ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse
from urllib.request import urlopen

from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import padding, rsa
import dioxus_account_audit as shared
import dioxus_analytics_audit as analytics

ROOT = shared.ROOT
BUNDLE = shared.BUNDLE
OUT = ROOT / 'target/dioxus-migration/analytics-session'
TOKEN = ''
REJECT_REFRESH = False
CAP = None
SLOW_LIMIT = None
REFRESHES = 0
AUTH_READS = []


class Backend(shared.Backend):
    def end_headers(self):
        if self.path == '/api/auth/session/refresh':
            self.send_header('x-epsx-refresh-outcome', 'rejected' if REJECT_REFRESH else 'rotated')
        super().end_headers()

    def do_GET(self):
        path = urlparse(self.path).path
        if path.endswith('/rankings'):
            query = parse_qs(urlparse(self.path).query)
            AUTH_READS.append(bool(self.headers.get('Authorization')))
            if query.get('limit') == [str(SLOW_LIMIT)]:
                time.sleep(2)
            if CAP:
                self.path = self.path.replace('limit=' + query.get('limit', ['10'])[0], 'limit=' + str(CAP))
            return analytics.Backend.do_GET(self)
        if path.endswith('/filters'):
            return analytics.Backend.do_GET(self)
        return super().do_GET()

    def do_POST(self):
        global REFRESHES
        body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))) or '{}')
        if self.path == '/api/auth/web3/challenge':
            return self.reply({'success': True, 'wallet_address': shared.WALLET,
                               'nonce': 'fixture-nonce', 'message': 'Fixture sign-in verification',
                               'expires_at': int(time.time()) + 300})
        if self.path in ['/api/auth/web3/verify', '/api/auth/session/refresh']:
            if self.path.endswith('/refresh'):
                REFRESHES += 1
                time.sleep(.3)
                if REJECT_REFRESH:
                    return self.reply({'error': 'expired refresh'}, 401)
            else:
                assert body['signature'] == '0xfixture-signature'
                assert body['client_id'] == 'epsx-frontend'
            return self.reply({'success': True, 'authenticated': True, 'wallet_address': shared.WALLET,
                               'permissions': [], 'access_token': TOKEN, 'refresh_token': 'fixture-refresh',
                               'expires_in': 600, 'refresh_expires_in': 3600,
                               'user': {'wallet_address': shared.WALLET, 'subject': shared.WALLET,
                                        'permissions': [], 'auth_method': 'web3_siwe'}})
        return self.reply({'error': 'unmatched fixture'}, 404)


def browser(*args):
    result = subprocess.run(['agent-browser', '--session', 'epsx-analytics-session', *args],
                            capture_output=True, text=True, timeout=40)
    if result.returncode:
        raise RuntimeError(f'{args[0]}: {result.stderr} {result.stdout}')
    return result.stdout.strip()


def ready(count):
    browser('wait', '--fn', f'document.querySelectorAll("[data-stock-card]").length === {count}'
            ' && document.querySelector("[data-dioxus-analytics]")?.getAttribute("aria-busy") === "false"')


def main():
    global TOKEN, REJECT_REFRESH, CAP, SLOW_LIMIT
    OUT.mkdir(parents=True, exist_ok=True)
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    public = key.public_key().public_numbers()
    shared.JWKS = {'keys': [{'kty': 'RSA', 'use': 'sig', 'alg': 'RS256', 'kid': 'session-fixture',
                            'n': shared.number(public.n), 'e': shared.number(public.e)}]}
    upstream = ThreadingHTTPServer(('127.0.0.1', 0), Backend)
    threading.Thread(target=upstream.serve_forever, daemon=True).start()
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    base = f'http://127.0.0.1:{port}'
    backend = f'http://127.0.0.1:{upstream.server_port}'
    now = int(time.time())
    header = shared.b64(json.dumps({'alg': 'RS256', 'kid': 'session-fixture'}).encode())
    claims = shared.b64(json.dumps({'iss': backend, 'sub': shared.WALLET, 'aud': ['epsx-frontend'],
                                    'exp': now + 600, 'iat': now - 1, 'jti': 'session-fixture',
                                    'scope': 'openid permissions', 'wallet_address': shared.WALLET,
                                    'auth_method': 'web3_siwe', 'auth_time': now - 1}).encode())
    unsigned = f'{header}.{claims}'
    TOKEN = unsigned + '.' + shared.b64(key.sign(unsigned.encode(), padding.PKCS1v15(), hashes.SHA256()))
    env = dict(os.environ, EPSX_ENV='development', ENV='development', IP='127.0.0.1', HOST='127.0.0.1',
               PORT=str(port), FRONTEND_URL=base, API_URL=backend, BACKEND_URL=backend,
               IDENTITY_SERVICE_URL=backend, CONTENT_SERVICE_URL=backend, NOTIFICATION_SERVICE_URL=backend,
               OIDC_ISSUER=backend, OIDC_JWKS_URL=backend + '/.well-known/jwks.json',
               DIOXUS_PUBLIC_PATH=str(BUNDLE / 'public'))
    records = []
    with (OUT / 'server.log').open('w') as log:
        binary = OUT / f'server-{os.getpid()}'
        shutil.copy2(BUNDLE / 'server', binary)
        process = subprocess.Popen([str(binary)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if process.poll() is not None:
                    raise RuntimeError('Fixture BFF exited')
                try:
                    with urlopen(base + '/api/health', timeout=.3):
                        break
                except OSError:
                    time.sleep(.1)
            else:
                raise RuntimeError('Fixture BFF did not start')
            browser('cookies', 'clear')
            browser('cookies', 'set', 'epsx.frontend.refresh_token', 'fixture-refresh', '--url', base, '--httpOnly')
            browser('open', base + '/analytics?page=2&limit=25&country=america')
            ready(25)
            assert REFRESHES == 1, REFRESHES
            assert AUTH_READS and all(AUTH_READS), AUTH_READS
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('eval', 'window.originalDocument=document')
            records.append('Refresh-only revisit recovers once before reading authenticated rankings')
            browser('select', '#analytics-limit', '50')
            ready(50)
            assert parse_qs(urlparse(browser('get', 'url')).query) == {'page': ['1'], 'limit': ['50'], 'country': ['america']}
            browser('click', '[aria-label="Next page"]')
            browser('wait', '--fn', 'location.search.includes("page=2") && document.querySelector("[data-stock-card]")?.getAttribute("data-rank") === "51"')
            assert browser('eval', 'document === originalDocument') == 'true'
            records.append('Page and limit controls work after recovery without a document reload')
            browser('cookies', 'set', 'epsx.frontend.access_token', 'expired', '--url', base, '--httpOnly')
            browser('select', '#analytics-limit', '25')
            ready(25)
            assert REFRESHES == 2, REFRESHES
            assert all(AUTH_READS), AUTH_READS
            records.append('Expiry while mounted retries the selected query with one refresh, never guest data')
            REJECT_REFRESH = True
            browser('cookies', 'set', 'epsx.frontend.access_token', 'expired', '--url', base, '--httpOnly')
            browser('select', '#analytics-limit', '100')
            browser('wait', '[data-session-state="sign-in-required"]')
            assert browser('eval', 'document.querySelectorAll("[data-stock-card]").length') == '0'
            assert 'limit=100' in browser('get', 'url')
            assert REFRESHES == 3, REFRESHES
            browser('click', '.epsx-session-primary')
            browser('wait', '.auth-card-cta button')
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('eval', f"window.ethereum={{request:async({{method}})=>{{if(['eth_requestAccounts','eth_accounts'].includes(method))return ['{shared.WALLET}'];if(method==='personal_sign')return '0xfixture-signature';throw Error(method);}}}}")
            browser('click', '.auth-card-cta button')
            ready(100)
            assert 'country=america' in browser('get', 'url')
            browser('select', '#analytics-limit', '10')
            ready(10)
            browser('back')
            ready(100)
            browser('forward')
            ready(10)
            records.append('Expired refresh removes stale cards, preserves filters through SIWE, and keeps limit/history working')
            SLOW_LIMIT = 25
            browser('select', '#analytics-limit', '25')
            browser('back')
            ready(10)
            time.sleep(2.2)
            assert browser('get', 'value', '#analytics-limit') == '10'
            assert browser('eval', 'document.querySelectorAll("[data-stock-card]").length') == '10'
            SLOW_LIMIT = None
            records.append('A delayed response cannot overwrite the previous page after Back navigation')
            CAP = 5
            browser('select', '#analytics-limit', '25')
            ready(5)
            assert browser('get', 'value', '#analytics-limit') == '5'
            browser('wait', '--text', 'Your current access allows up to 5 companies per page.')
            # Selecting the same over-cap value again must still settle at 5.
            browser('select', '#analytics-limit', '25')
            ready(5)
            assert browser('get', 'value', '#analytics-limit') == '5'
            records.append('Selector reflects the backend cap even across repeated over-cap selections')
            CAP = None
            browser('cookies', 'set', 'epsx.frontend.access_token', 'expired', '--url', base, '--httpOnly')
            browser('cookies', 'set', 'epsx.frontend.refresh_token', 'expired-refresh', '--url', base, '--httpOnly')
            browser('open', base + '/analytics?page=3&limit=25&country=america')
            browser('wait', '[data-session-state="sign-in-required"]')
            browser('wait', '--fn', 'document.querySelector("[data-dioxus-analytics]")?.getAttribute("aria-busy") === "false"')
            assert REFRESHES == 4, REFRESHES
            assert browser('eval', 'document.querySelectorAll("[data-stock-card]").length') == '0'
            assert 'page=3' in browser('get', 'url')
            records.append('Reopening with both tokens expired never silently falls back to public rankings')
            browser('cookies', 'clear')
            browser('open', base + '/analytics?page=1&limit=10')
            ready(10)
            assert REFRESHES == 4, REFRESHES
            assert AUTH_READS[-1] is False
            records.append('An intentional signed-out visit still supports public pagination')
            errors = browser('errors')
            assert 'panic' not in errors.lower(), errors
            (OUT / 'report.json').write_text(json.dumps(records, indent=2))
            print(json.dumps(records, indent=2))
        except Exception:
            (OUT / 'failure.txt').write_text(browser('snapshot') + '\n' + browser('errors'))
            (OUT / 'progress.json').write_text(json.dumps({'records': records, 'refreshes': REFRESHES, 'authenticated_reads': AUTH_READS}, indent=2))
            raise
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
            upstream.shutdown()
            binary.unlink(missing_ok=True)


if __name__ == '__main__':
    main()
