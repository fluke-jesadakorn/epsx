"""Logout through a Host-rewriting proxy with isolated sessions and a fake wallet.

No user wallet, account, database, or production service is used.
"""
import http.client
import json
import os
import shutil
import socket
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.request import Request, urlopen

from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import padding, rsa
import dioxus_analytics_session_audit as fixture

OUT = fixture.ROOT / 'target/dioxus-migration/logout'
REVOKE_STATUS = 200
REVOCATIONS = []


class Backend(fixture.Backend):
    def do_DELETE(self):
        assert self.path == '/api/auth/web3/logout'
        body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', 0))) or '{}')
        REVOCATIONS.append(body)
        self.reply({'success': REVOKE_STATUS == 200}, REVOKE_STATUS)


class Proxy(BaseHTTPRequestHandler):
    """Reproduce DX changing Host while preserving the browser's Origin."""
    target_port = 0

    def log_message(self, *_):
        pass

    def forward(self):
        body = self.rfile.read(int(self.headers.get('Content-Length', 0)))
        headers = dict(self.headers)
        headers['Host'] = f'127.0.0.1:{self.target_port}'
        headers['Connection'] = 'close'
        connection = http.client.HTTPConnection('127.0.0.1', self.target_port, timeout=30)
        try:
            connection.request(self.command, self.path, body, headers)
            response = connection.getresponse()
            payload = response.read()
            self.send_response(response.status)
            for key, value in response.getheaders():
                if key.lower() not in ['connection', 'transfer-encoding', 'content-length']:
                    self.send_header(key, value)
            self.send_header('Content-Length', str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        finally:
            connection.close()

    do_GET = forward
    do_POST = forward


def browser(*args):
    result = subprocess.run(['agent-browser', '--session', 'epsx-logout-audit', *args],
                            capture_output=True, text=True, timeout=40)
    if result.returncode:
        raise RuntimeError(f'{args[0]}: {result.stderr} {result.stdout}')
    return result.stdout.strip()


def main():
    global REVOKE_STATUS
    OUT.mkdir(parents=True, exist_ok=True)
    key = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    public = key.public_key().public_numbers()
    shared = fixture.shared
    shared.JWKS = {'keys': [{'kty': 'RSA', 'use': 'sig', 'alg': 'RS256', 'kid': 'logout-fixture',
                            'n': shared.number(public.n), 'e': shared.number(public.e)}]}
    upstream = ThreadingHTTPServer(('127.0.0.1', 0), Backend)
    threading.Thread(target=upstream.serve_forever, daemon=True).start()
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    Proxy.target_port = port
    proxy = ThreadingHTTPServer(('127.0.0.1', 0), Proxy)
    threading.Thread(target=proxy.serve_forever, daemon=True).start()
    base = f'http://127.0.0.1:{proxy.server_port}'
    backend = f'http://127.0.0.1:{upstream.server_port}'
    now = int(time.time())
    header = shared.b64(json.dumps({'alg': 'RS256', 'kid': 'logout-fixture'}).encode())
    claims = shared.b64(json.dumps({'iss': backend, 'sub': shared.WALLET, 'aud': ['epsx-frontend'],
                                    'exp': now + 600, 'iat': now - 1, 'jti': 'logout-fixture',
                                    'scope': 'openid permissions', 'wallet_address': shared.WALLET,
                                    'auth_method': 'web3_siwe', 'auth_time': now - 1}).encode())
    unsigned = f'{header}.{claims}'
    fixture.TOKEN = unsigned + '.' + shared.b64(key.sign(unsigned.encode(), padding.PKCS1v15(), hashes.SHA256()))
    env = dict(os.environ, EPSX_ENV='development', ENV='development', IP='127.0.0.1', HOST='127.0.0.1',
               PORT=str(port), FRONTEND_URL=base, API_URL=backend, BACKEND_URL=backend,
               IDENTITY_SERVICE_URL=backend, CONTENT_SERVICE_URL=backend, NOTIFICATION_SERVICE_URL=backend,
               OIDC_ISSUER=backend, OIDC_JWKS_URL=backend + '/.well-known/jwks.json',
               DIOXUS_PUBLIC_PATH=str(fixture.BUNDLE / 'public'))
    records = []
    with (OUT / 'server.log').open('w') as log:
        binary = OUT / f'server-{os.getpid()}'
        shutil.copy2(fixture.BUNDLE / 'server', binary)
        process = subprocess.Popen([str(binary)], cwd=fixture.ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if process.poll() is not None:
                    raise RuntimeError('Fixture BFF exited')
                try:
                    with urlopen(f'http://127.0.0.1:{port}/api/health', timeout=.3):
                        break
                except OSError:
                    time.sleep(.1)
            else:
                raise RuntimeError('Fixture BFF did not start')
            for headers in [{}, {'Origin': 'https://evil.example'},
                            {'Origin': base, 'Sec-Fetch-Site': 'cross-site'}]:
                request = Request(base + '/_server/frontend/logout', data=b'{}',
                                  headers={'Content-Type': 'application/json', **headers})
                with urlopen(request, timeout=10) as response:
                    assert json.load(response) == {'Err': 'Forbidden'}
                    assert not response.headers.get_all('Set-Cookie')
            assert not REVOCATIONS
            records.append('Missing/foreign Origin and cross-site requests cannot clear cookies or revoke sessions')

            for status, expired in [(200, False), (401, True), (503, False)]:
                REVOKE_STATUS = status
                browser('cookies', 'clear')
                browser('cookies', 'set', 'epsx.frontend.access_token', fixture.TOKEN, '--url', base, '--httpOnly')
                browser('cookies', 'set', 'epsx.frontend.refresh_token', 'fixture-refresh', '--url', base, '--httpOnly')
                browser('open', base + '/analytics?page=1&limit=10')
                browser('wait', '[data-dioxus-hydrated="true"]')
                browser('wait', '.fe-wallet-trigger')
                if expired:
                    browser('cookies', 'set', 'epsx.frontend.access_token', 'expired', '--url', base, '--httpOnly')
                browser('eval', "window.ethereum={request:async()=>{throw Error('Fixture: extension does not support revocation');}}")
                browser('click', '.fe-wallet-trigger')
                browser('click', '.fe-wallet-disconnect')
                browser('wait', '--fn', 'location.pathname === "/" && !!document.querySelector(".fe-connect-wallet")')
                cookies = browser('cookies', 'get')
                assert 'epsx.frontend.access_token' not in cookies and 'epsx.frontend.refresh_token' not in cookies
                assert 'Could not disconnect' not in browser('get', 'text', 'body')
                browser('open', base + '/analytics?page=1&limit=10')
                browser('wait', '[data-dioxus-hydrated="true"]')
                browser('wait', '.fe-connect-wallet')
                assert not fixture.REFRESHES, fixture.REFRESHES
                records.append(f'Proxy logout clears cookies and stays signed out after reload (upstream {status}, expired access {expired})')
            assert len(REVOCATIONS) == 3, REVOCATIONS
            assert all(item.get('refresh_token') == 'fixture-refresh' for item in REVOCATIONS)

            browser('open', base + '/auth?return_url=%2Fanalytics%3Fpage%3D2%26limit%3D25')
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('eval', f"window.ethereum={{request:async({{method}})=>{{if(['eth_requestAccounts','eth_accounts'].includes(method))return ['{shared.WALLET}'];if(method==='personal_sign')return '0xfixture-signature';throw Error(method);}}}}")
            browser('click', '.auth-card-cta button')
            browser('wait', '--fn', 'document.querySelectorAll("[data-stock-card]").length === 25 && !!document.querySelector(".fe-wallet-trigger")')
            assert 'page=2&limit=25' in browser('get', 'url')
            assert 'panic' not in browser('errors').lower()
            records.append('Wallet sign-in after disconnect succeeds and preserves the requested page and limit')
            (OUT / 'report.json').write_text(json.dumps(records, indent=2))
            print(json.dumps(records, indent=2))
        except Exception:
            (OUT / 'failure.txt').write_text(browser('snapshot') + '\n' + browser('errors'))
            raise
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
            proxy.shutdown()
            upstream.shutdown()
            binary.unlink(missing_ok=True)
            browser('close')


if __name__ == '__main__':
    main()
