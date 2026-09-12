"""Loopback-only UI fixtures layered on the repository's Rust E2E authority.

The historical migration fixture predates the current filters and grouped
watchlist projections. This adapter adds those contracts for frontend QA;
all other requests still reach the Rust fixture. Never use with real data.
"""
from http.server import BaseHTTPRequestHandler
from urllib.request import Request, urlopen
from urllib.parse import urlsplit, parse_qs, urlencode
from urllib.error import HTTPError
import json
import uuid
import os
import copy
import time
from premium_fixtures import read_fixture

SOURCE = 'http://127.0.0.1:48081'
FAULTS = set()
SYMBOLS = ['NVDA', 'MSFT']
GROUPS = [{'id': '550e8400-e29b-41d4-a716-446655440010', 'name': 'Research', 'position': 0, 'symbols': ['NVDA']}]



def premium_rankings(body, query):
    """Test-only density/history scenarios. Never an application data fallback."""
    value = json.loads(body)
    if not value.get('success') or not value.get('data') or value.get('access_info', {}).get('locked_ranks_count', 0):
        return body
    if value['data'][0].get('symbol') != 'NVDA':
        return body
    rows = copy.deepcopy(value['data'])
    for index in range(9):
        row = copy.deepcopy(rows[index % 3])
        row.update(symbol=f'TEST{index+1:02}', company_name=[
            'Example Research & Development Corporation',
            'Example Industrial Holdings', 'Example Healthcare Group'][index % 3],
            current_eps=round(2.2-index*.23, 2), price_current=round(94-index*4.2, 2))
        row['quarterly_performance'][0].update(eps=row['current_eps'], eps_growth=round(12-index*3.2, 2))
        rows.append(row)
    for row in rows:
        current = row['quarterly_performance'][0]
        for date, quarter, factor in [('2026-03-31','Q1 2026',.85),('2025-12-31','Q4 2025',.89),('2025-09-30','Q3 2025',.72),('2025-06-30','Q2 2025',.68)]:
            earlier = copy.deepcopy(current)
            earlier.update(date=date, quarter=quarter, eps=round(current['eps']*factor,2), announcement_date=None, announcement_timestamp=None)
            row['quarterly_performance'].append(earlier)
    if os.environ.get('EPSX_QA_NEXT_ACTION') == '1':
        from datetime import datetime, timezone
        def stamp(day): return int(datetime.fromisoformat(day).replace(tzinfo=timezone.utc).timestamp())
        today = datetime.now(timezone.utc).date().isoformat()
        cases = [
            (stamp('2026-11-26'), stamp('2026-08-27')),
            (None, stamp('2026-07-30')),
            (None, None),
            (stamp(today), stamp('2026-06-01')),
            (stamp('2026-09-01'), None),
            (9223372036854775807, stamp('2026-08-01')),
            (None, stamp('2026-01-01')),
            (0, -1),
            (stamp('2026-12-10'), None),
        ]
        for index, row in enumerate(rows):
            row['next_earnings_date'], row['last_earnings_date'] = cases[index % len(cases)]
    params = parse_qs(query)
    sort = params.get('sort_by',['eps_growth'])[0]
    keys = {'current_eps':lambda r:r['current_eps'], 'price':lambda r:r['price_current'],
            'symbol':lambda r:r['symbol'], 'name':lambda r:r['company_name'],
            'eps_growth':lambda r:r['quarterly_performance'][0]['eps_growth']}
    rows.sort(key=keys.get(sort,keys['eps_growth']), reverse=sort not in ('symbol','name'))
    for index,row in enumerate(rows): row['rank']=index+1
    limit = max(1,min(100,int(params.get('limit',['10'])[0])))
    page = max(1,int(params.get('page',['1'])[0]))
    total=len(rows)
    value.update(data=rows[(page-1)*limit:page*limit],
        pagination={'page':page,'limit':limit,'total':total,'totalPages':(total+limit-1)//limit,
                    'hasNext':page*limit<total,'hasPrev':page>1})
    value['metadata']['data_source']='epsx-rust-e2e-fixture-premium'
    return json.dumps(value).encode()

def layout():
    grouped = {symbol for group in GROUPS for symbol in group['symbols']}
    return {'groups': GROUPS, 'ungrouped': [symbol for symbol in SYMBOLS if symbol not in grouped], 'watched': len(SYMBOLS)}


class Handler(BaseHTTPRequestHandler):
    def do_GET(self): self.handle_request()
    def do_POST(self): self.handle_request()
    def do_PUT(self): self.handle_request()
    def do_DELETE(self): self.handle_request()

    def send_json(self, data, status=200):
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Cache-Control', 'no-store')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def handle_request(self):
        path = urlsplit(self.path).path
        raw = self.rfile.read(int(self.headers.get('Content-Length', '0')))
        data = json.loads(raw or '{}')
        if path == '/__qa/state' and os.environ.get('EPSX_QA_PREMIUM') == '1':
            if self.headers.get('x-epsx-e2e-token') != 'epsx-enterprise-local-fixture':
                return self.send_json({'error':'unauthorized'},401)
            if self.command == 'PUT':
                FAULTS.clear()
                FAULTS.update(set(data.get('faults',[])) & {'filters-unavailable','watchlist-unavailable','watchlist-delayed','watchlist-unacknowledged','rankings-restricted'})
            return self.send_json({'faults':sorted(FAULTS)})
        if path == '/api/analytics/rankings' and 'rankings-restricted' in FAULTS:
            return self.send_json({'error':'forbidden'},403)
        if path == '/api/analytics/filters' and 'filters-unavailable' in FAULTS:
            return self.send_json({'error':'unavailable'},503)
        if path.startswith('/api/users/watchlist') and self.command != 'GET' and 'watchlist-unavailable' in FAULTS:
            time.sleep(1)
            return self.send_json({'error':'unavailable'},503)
        if path.startswith('/api/users/watchlist') and self.command != 'GET' and 'watchlist-delayed' in FAULTS:
            time.sleep(1)
        if path.startswith('/api/users/watchlist') and self.command != 'GET' and 'watchlist-unacknowledged' in FAULTS:
            if path == '/api/users/watchlist/layout':
                return self.send_json({'success': True, 'data': layout()})
            return self.send_json({'success': True, 'data': {'symbols': SYMBOLS}})
        if os.environ.get("EPSX_QA_PREMIUM") == "1" and self.command == "GET":
            fixture = read_fixture(path, urlsplit(self.path).query)
            if fixture is not None:
                if path.startswith('/api/public/') or path == '/api-docs/openapi.json' or self.headers.get('Authorization', '').startswith('Bearer '):
                    return self.send_json(fixture)
                return self.send_json({'error': 'authentication_required'}, 401)
        if path == '/api/analytics/filters':
            return self.send_json({'countries': [{'label': 'United States', 'value': 'america'}], 'sectors': ['Technology'], 'exchanges': ['NASDAQ'], 'stock_types': ['common']})
        if path.startswith('/api/users/watchlist'):
            if not self.headers.get('Authorization', '').startswith('Bearer '):
                return self.send_json({'error': 'authentication_required'}, 401)
            if path == '/api/users/watchlist':
                if self.command == 'POST':
                    symbol = data['symbol'].upper()
                    if symbol not in SYMBOLS: SYMBOLS.append(symbol)
                    for group in GROUPS:
                        if group['id'] in data.get('group_ids', []) and symbol not in group['symbols']: group['symbols'].append(symbol)
                return self.send_json({'success': True, 'data': {'symbols': SYMBOLS}})
            if path in ('/api/users/watchlist/layout', '/api/users/watchlist/groups'):
                if self.command == 'POST':
                    GROUPS.append({'id': str(uuid.uuid4()), 'name': data['name'], 'position': len(GROUPS), 'symbols': []})
                elif self.command == 'PUT':
                    by_id = {group['id']: group for group in GROUPS}
                    for position, value in enumerate(data['groups']):
                        by_id[value['id']]['symbols'] = value['symbols']
                        by_id[value['id']]['position'] = position
                    GROUPS.sort(key=lambda group: group['position'])
                    # Preserve the submitted ungrouped order as the real owner
                    # layout endpoint does; do not silently acknowledge a no-op.
                    grouped = [symbol for group in GROUPS for symbol in group['symbols']]
                    SYMBOLS[:] = list(dict.fromkeys(data['ungrouped'] + grouped))
                return self.send_json({'success': True, 'data': layout()})
            if path.startswith('/api/users/watchlist/groups/'):
                group_id = path.rsplit('/', 1)[-1]
                if self.command == 'DELETE': GROUPS[:] = [group for group in GROUPS if group['id'] != group_id]
                else:
                    for group in GROUPS:
                        if group['id'] == group_id: group['name'] = data['name']
                for position, group in enumerate(GROUPS): group['position'] = position
                return self.send_json({'success': True, 'data': layout()})
            if self.command == 'DELETE':
                symbol = path.rsplit('/', 1)[-1].upper()
                SYMBOLS[:] = [value for value in SYMBOLS if value != symbol]
                for group in GROUPS: group['symbols'] = [value for value in group['symbols'] if value != symbol]
                return self.send_json({'success': True, 'data': {'symbols': SYMBOLS}})
        headers = {k: v for k, v in self.headers.items() if k.lower() not in ('host', 'connection', 'accept-encoding', 'content-length')}
        request_path = self.path
        if os.environ.get('EPSX_QA_PREMIUM') == '1' and path == '/api/analytics/rankings':
            params = parse_qs(urlsplit(self.path).query)
            params.update(page=['1'],limit=['100'])
            request_path = path + '?' + urlencode(params,doseq=True)
        request = Request(SOURCE + request_path, data=raw if raw else None, headers=headers, method=self.command)
        try: response = urlopen(request, timeout=15)
        except HTTPError as error: response = error
        body = response.read()
        if os.environ.get("EPSX_QA_PREMIUM") == "1" and path == "/api/analytics/rankings" and response.status == 200:
            body = premium_rankings(body, urlsplit(self.path).query)
        self.send_response(response.status)
        self.send_header('Content-Type', response.headers.get('Content-Type', 'application/json'))
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args): pass


if __name__ == '__main__':
    print('Enterprise fixture adapter: http://127.0.0.1:48082', flush=True)
    # A single server thread keeps this small stateful fixture deterministic.
    from http.server import HTTPServer
    HTTPServer(('127.0.0.1', 48082), Handler).serve_forever()
