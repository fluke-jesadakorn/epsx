"""Read-only inventory of production pages and the API references in their assets.

Fetches only known frontend routes and script URLs published in their HTML.
Does not authenticate, invoke API mutations, or change production state.
"""
import concurrent.futures
import hashlib
import json
import re
import urllib.error
import urllib.parse
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/prod-page-parity/production'
ORIGIN = 'https://epsx.io'
ROUTES = [
    '/', '/analytics', '/portfolio', '/plans', '/pricing', '/news', '/about',
    '/contact', '/auth', '/dashboard', '/account', '/account/credits', '/profile',
    '/permissions', '/notifications', '/payment', '/chat', '/chat/history',
    '/developer', '/developer/usage', '/developer/docs', '/privacy', '/terms',
    '/offline', '/access-denied', '/manual',
]


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.scripts = set()
        self.links = set()
        self.title = ''
        self.in_title = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'script' and attrs.get('src', '').startswith('/_next/'):
            self.scripts.add(attrs['src'])
        if tag == 'a' and attrs.get('href', '').startswith('/'):
            self.links.add(attrs['href'])
        self.in_title |= tag == 'title'

    def handle_endtag(self, tag):
        if tag == 'title':
            self.in_title = False

    def handle_data(self, data):
        if self.in_title:
            self.title += data


def fetch(url):
    request = urllib.request.Request(url, headers={'User-Agent': 'EPSX-route-parity-review/1.0'})
    try:
        with urllib.request.urlopen(request, timeout=25) as response:
            return response.status, response.url, response.read().decode('utf-8', 'replace')
    except urllib.error.HTTPError as error:
        return error.code, error.url, error.read().decode('utf-8', 'replace')
    except (OSError, TimeoutError) as error:
        return None, url, str(error)


def page(path):
    status, final_url, body = fetch(ORIGIN + path)
    filename = (path.strip('/').replace('/', '-') or 'home') + '.html'
    (OUT / filename).write_text(body)
    parsed = Page()
    parsed.feed(body)
    return dict(path=path, status=status, final_url=final_url, title=parsed.title,
                scripts=sorted(parsed.scripts), links=sorted(parsed.links), file=filename)


def asset(path):
    status, final_url, body = fetch(ORIGIN + path)
    filename = path.rsplit('/', 1)[-1]
    (OUT / 'assets' / filename).write_text(body)
    # References are evidence, not proof that a server currently implements them.
    api = sorted(set(re.findall(r'''["'`](/(?:api(?:-docs)?|v1)/[^"'`\s<>\\]*)''', body)))
    paths = sorted(set(re.findall(r'''(?:href|pathname|path):["'`](/[^"'`\s<>\\]*)''', body)))
    return dict(path=path, status=status, file='assets/' + filename,
                sha256=hashlib.sha256(body.encode()).hexdigest(), api_references=api,
                page_references=paths)


def main():
    (OUT / 'assets').mkdir(parents=True, exist_ok=True)
    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        pages = list(pool.map(page, ROUTES))
        scripts = sorted({script for item in pages for script in item['scripts']})
        assets = list(pool.map(asset, scripts))
    result = dict(origin=ORIGIN, pages=pages, assets=assets,
                  api_references=sorted({ref for item in assets for ref in item['api_references']}),
                  page_references=sorted({ref for item in assets for ref in item['page_references']}))
    (OUT / 'inventory.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: result[key] for key in ['api_references', 'page_references']}, indent=2))
    print(f'Captured {len(pages)} routes and {len(assets)} public scripts in {OUT}')


if __name__ == '__main__':
    main()
