"""Local QA only: attach a short-lived Rust fixture session to the preview.

Binds exclusively to loopback and talks exclusively to fixed loopback ports.
No production credentials or live wallet signing are used.
"""
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.request import Request, urlopen
from urllib.error import HTTPError
import json

SESSION = json.load(urlopen(Request(
    'http://127.0.0.1:48081/__e2e/session?audience=epsx-frontend&permissions=epsx%3Aanalytics%3Aread',
    headers={'x-epsx-e2e-token':'epsx-enterprise-local-fixture'},
), timeout=10))['accessToken']


class Handler(BaseHTTPRequestHandler):
    def do_GET(self): self.forward()
    def do_POST(self): self.forward()
    def do_PUT(self): self.forward()
    def do_DELETE(self): self.forward()

    def forward(self):
        headers = {k:v for k,v in self.headers.items() if k.lower() not in ('connection','accept-encoding','cookie')}
        headers['Cookie'] = 'epsx.frontend.access_token=' + SESSION
        data = self.rfile.read(int(self.headers.get('Content-Length','0'))) if self.command not in ('GET','HEAD') else None
        request = Request('http://127.0.0.1:3300'+self.path, data=data, headers=headers, method=self.command)
        try: response = urlopen(request, timeout=20)
        except HTTPError as error: response = error
        body = response.read()
        self.send_response(response.status)
        for key,value in response.headers.items():
            if key.lower() not in ('transfer-encoding','connection','content-length','content-encoding'):
                self.send_header(key,value)
        self.send_header('Content-Length',str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self,*args): pass


if __name__ == '__main__':
    print('Authenticated fixture preview: http://127.0.0.1:3301', flush=True)
    ThreadingHTTPServer(('127.0.0.1',3301),Handler).serve_forever()
