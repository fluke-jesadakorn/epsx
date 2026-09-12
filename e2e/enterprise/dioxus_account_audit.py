"""Authenticated SSR/WASM purchases + account audit with ephemeral RSA/JWKS.
Build dx-frontend fullstack first. Uses no production service or user credentials.
"""
import base64
import json
import os
from pathlib import Path
import socket
import subprocess
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse, parse_qs
from urllib.request import urlopen, Request
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.primitives import hashes

ROOT = Path(__file__).resolve().parents[2]
BUNDLE = Path(os.environ.get('EPSX_AUDIT_FRONTEND_BUNDLE', str(ROOT / 'target/dx/dx-frontend/debug/web')))
OUT = ROOT / 'target/dioxus-migration/account'
WALLET = '0x1111111111111111111111111111111111111111'
ORDER = '11111111-1111-4111-8111-111111111111'
PREFERENCES = {'channels': {'email': True, 'in_app': True, 'push': False}, 'quiet_hours': {'start': '22:00', 'end': '07:00', 'enabled': False}, 'timezone': 'UTC', 'updated_at': '2026-09-09T00:00:00Z'}
READS = []
WRITES = []
FAIL_PURCHASES = False
JWKS = {}
COUNTERS = {"jwks": 0}
WATCH = {"groups": [], "ungrouped": ["AAPL"], "watched": 1}
GROUP = "10000000-0000-4000-8000-000000000001"
TOPIC = "20000000-0000-4000-8000-000000000001"
CONVERSATION = "30000000-0000-4000-8000-000000000001"
CHAT = []
MESSAGES = []
CHAT_CREATES = []
UPLOADS = []
FAIL_UPLOAD = True
NOTIFICATION_READ = False
NOTIFICATION_TITLE = "Fixture notification"
NOTIFICATION_EVENT = threading.Event()
NOTIFICATION_STREAMS = []
FAIL_DEVELOPER_CREATE=True
DEVELOPER_KEYS=[]
DEVELOPER_WRITES=[]
DEVELOPER_SECRET='epsx_'+'a'*64
DEVELOPER_KEY_ID='50000000-0000-4000-8000-000000000001'
DEVELOPER_TRIES=[]
def developer_spec():
    return {'openapi':'3.0.0','info':{'title':'Fixture API','version':'1'},'paths':{'/api/fixture/read':{'get':{'operationId':'fixtureRead','summary':'Fixture read','x-epsx-required-scopes':['epsx:analytics:view'],'x-epsx-api-key-callable':True,'x-epsx-mutation':False,'x-epsx-idempotent':False,'responses':{'200':{'description':'Success'}}}}}}
STAMP = "2026-09-09T00:00:00Z"
def chat_topic(): return {'id':TOPIC,'name':'general','label':'General','description':'General questions','icon':'message-circle','sort_order':0,'is_active':True,'created_at':STAMP}
def chat_message(content): return {'id':f'40000000-0000-4000-8000-{len(MESSAGES)+1:012d}','conversation_id':CONVERSATION,'sender_type':'user','sender_address':WALLET,'content':content,'is_read':False,'metadata':{},'created_at':STAMP}


def b64(value): return base64.urlsafe_b64encode(value).decode().rstrip('=')
def number(value): return b64(value.to_bytes((value.bit_length() + 7) // 8, 'big'))

def order():
    return {'order_id': ORDER, 'plan_name': 'Fixture day plan', 'wallet_address': WALLET,
            'amount': '5000000', 'token': 'USDT', 'token_decimals': 6, 'status': 'succeeded',
            'payment_status': 'succeeded', 'fulfillment_status': 'expired', 'created_at': '2026-09-09T03:37:17Z',
            'payment_id': None, 'contract_address': None, 'payment_available': True, 'tx_hash': None}

class Backend(BaseHTTPRequestHandler):
    def log_message(self, *_): pass
    def reply(self, body, code=200):
        self.send_response(code)
        self.send_header('Content-Type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(body).encode())
    def do_GET(self):
        path = urlparse(self.path).path
        if path.endswith('/jwks.json'):
            COUNTERS['jwks'] += 1
            return self.reply(JWKS)
        if path == '/api-docs/openapi.json': return self.reply(developer_spec())
        if path == '/api/fixture/read':
            DEVELOPER_TRIES.append(dict(self.headers))
            assert self.headers.get('Authorization')=='Bearer '+DEVELOPER_SECRET
            return self.reply({'fixture_result':'success'})
        if not self.headers.get('Authorization', '').startswith('Bearer '): return self.reply({'error': 'missing bearer'}, 401)
        READS.append(path)
        if path == '/api/developer-portal/overview':
            days=int(parse_qs(urlparse(self.path).query).get('days',['30'])[0])
            entitlement={'plans':[],'assignable_scopes':['epsx:analytics:view'],'rate_limits':{'per_minute':10,'per_hour':100,'per_day':1000,'burst':5},'can_read':True,'can_write':True,'has_active_api_entitlement':True}
            usage={'days':days,'total_requests':0,'successful_requests':0,'error_requests':0,'success_rate':100,'error_rate':0,'average_response_time_ms':0,'daily':[{'date':'2026-09-09','total_requests':0,'error_requests':0} for _ in range(days)],'top_endpoints':[]}
            return self.reply({'success':True,'data':{'entitlement':entitlement,'api_keys':DEVELOPER_KEYS,'total_api_keys':len(DEVELOPER_KEYS),'usage':usage},'error':None})
        if path == '/api/v1/notification/stream':
            NOTIFICATION_STREAMS.append(self.path)
            self.send_response(200); self.send_header('Content-Type','text/event-stream'); self.end_headers()
            self.wfile.write(b': connected\n\n'); self.wfile.flush()
            if NOTIFICATION_EVENT.wait(25):
                try: self.wfile.write(b'event: notification\nid: fixture-event\ndata: {}\n\n'); self.wfile.flush()
                except (BrokenPipeError,ConnectionResetError): pass
            return
        if path == '/api/v1/notification/list':
            item=dict(id='fixture-notification',user_id=WALLET,channel='in_app',recipient=WALLET,template_id=None,subject=None,body='Fixture account update',data=None,status='sent',error=None,sent_at=STAMP,created_at=STAMP,read_at=STAMP if NOTIFICATION_READ else None,clicked_at=None,title=NOTIFICATION_TITLE,notification_type='system',priority='normal',action_url=None,expires_at=None)
            return self.reply({'items':[item],'total':1})
        if path.startswith('/api/payments/pay-orders'):
            if FAIL_PURCHASES: return self.reply({'error': 'fixture outage'}, 503)
            return self.reply(order() if path.endswith(ORDER) else {'orders': [order()], 'next_offset': None})
        if path == '/api/chat/inbox': return self.reply({'success':True,'data':{'topics':[chat_topic()],'conversations':CHAT},'error':None})
        if path == f'/api/chat/conversations/{CONVERSATION}/full': return self.reply({'success':True,'data':{'conversation':CHAT[0],'messages':MESSAGES},'error':None})
        if path == '/api/users/watchlist/layout': return self.reply({'success':True,'data':WATCH,'error':None})
        if path == '/api/users/watchlist': return self.reply({'success':True,'data':{'symbols':list(dict.fromkeys(WATCH['ungrouped']+[symbol for group in WATCH['groups'] for symbol in group['symbols']]))},'error':None})
        if path == '/api/users/profile': return self.reply({'success': True, 'data': {'wallet_address': WALLET, 'permissions': [], 'auth_method': 'web3_siwe', 'created_at': '2026-01-01T00:00:00Z', 'last_login': '2026-09-09T00:00:00Z'}})
        if path == '/api/users/access-overview': return self.reply({'success': True, 'data': {'current_tier': 'basic', 'groups': [], 'direct_permissions': []}})
        if path == '/api/payments/credits/balance': return self.reply({'wallet_address': WALLET, 'balance': '12.5', 'pending_balance': '0', 'available_balance': '12.5', 'lifetime_earned': '20', 'lifetime_spent': '7.5', 'last_transaction_at': None})
        if path == '/api/payments/credits/history': return self.reply({'success': True, 'data': [], 'count': 0})
        if path == '/api/payments/history': return self.reply({'success': True, 'data': {'payments': [], 'pagination': {'page': 1, 'per_page': 10, 'total': 0, 'total_pages': 0}}})
        if path == f'/api/v1/pay/history/{WALLET}': return self.reply({'address': WALLET, 'intents': [], 'escrows': [], 'total_intents': 0, 'total_escrows': 0})
        if path == '/api/v1/notification/preferences': return self.reply(PREFERENCES)
        if path == '/api/v1/notification/push': return self.reply({'enabled': False, 'subscribed': False, 'public_key': None})
        return self.reply({'error': 'unmatched fixture route'}, 404)
    def do_POST(self):
        global NOTIFICATION_READ, FAIL_DEVELOPER_CREATE
        path = urlparse(self.path).path
        raw = self.rfile.read(int(self.headers.get('Content-Length','0')))
        if path.endswith('/upload'):
            UPLOADS.append(len(raw))
            return self.reply({'error':'fixture upload outage'},503) if FAIL_UPLOAD else self.reply({'success':True,'data':{},'error':None})
        if path == '/api/v1/notification/fixture-notification/read':
            NOTIFICATION_READ=True
            return self.reply({'ok':True})
        if path == '/api/v1/notification/fixture-notification/unread':
            NOTIFICATION_READ=False
            return self.reply({'ok':True})
        body = json.loads(raw)
        if path == '/api/developer-portal/my-keys':
            DEVELOPER_WRITES.append((path,body,self.headers.get('idempotency-key')))
            if FAIL_DEVELOPER_CREATE:
                FAIL_DEVELOPER_CREATE=False
                return self.reply({'error':'fixture outage'},503)
            key={'id':DEVELOPER_KEY_ID,'key_prefix':'epsx_aaaa…','name':body['name'],'description':body.get('description'),'status':'active','scopes':body['scopes'],'total_requests':0,'expires_at':None,'last_used_at':None,'created_at':STAMP}
            DEVELOPER_KEYS.append(key)
            return self.reply({'success':True,'data':{'api_key':key,'secret':DEVELOPER_SECRET,'replayed':False},'error':None})
        if path == f'/api/developer-portal/my-keys/{DEVELOPER_KEY_ID}/revoke':
            DEVELOPER_WRITES.append((path,body,self.headers.get('idempotency-key')))
            DEVELOPER_KEYS[0]['status']='revoked'
            return self.reply({'success':True,'data':{'id':DEVELOPER_KEY_ID,'status':'revoked','replayed':False},'error':None})
        if path == '/api/chat/conversations':
            CHAT_CREATES.append(body)
            CHAT.append({'id':CONVERSATION,'topic_id':body['topic_id'],'wallet_address':WALLET,'subject':body['subject'],'status':'open','assigned_agent':None,'last_message_at':STAMP,'unread_user':0,'unread_agent':1,'created_at':STAMP,'updated_at':STAMP,'metadata':{}})
            MESSAGES.append(chat_message(body['message']))
            return self.reply({'success':True,'data':CHAT[-1],'error':None})
        if path.endswith('/messages'):
            MESSAGES.append(chat_message(body['content']))
            return self.reply({'success':True,'data':MESSAGES[-1],'error':None})
        if path == '/api/users/watchlist/groups':
            WATCH['groups'].append({'id':GROUP,'name':body['name'],'position':len(WATCH['groups']),'symbols':[]})
            return self.reply({'success':True,'data':WATCH,'error':None})
        if path == '/api/users/watchlist':
            symbol=body['symbol']
            if symbol not in WATCH['ungrouped']: WATCH['ungrouped'].append(symbol)
            WATCH['watched']=len(set(WATCH['ungrouped']+[symbol for group in WATCH['groups'] for symbol in group['symbols']]))
            return self.reply({'success':True,'data':{'symbols':WATCH['ungrouped']},'error':None})
        return self.reply({'error':'unmatched fixture route'},404)
    def do_PUT(self):
        path=urlparse(self.path).path
        if path == '/api/users/watchlist/layout':
            body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            old={group['id']:group for group in WATCH['groups']}
            WATCH['groups']=[dict(old[group['id']],position=index,symbols=group['symbols']) for index,group in enumerate(body['groups'])]
            WATCH['ungrouped']=body['ungrouped']
            return self.reply({'success':True,'data':WATCH,'error':None})

        if urlparse(self.path).path == '/api/v1/notification/preferences':
            body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
            WRITES.append(body)
            PREFERENCES.update(body)
            return self.reply(PREFERENCES)
        return self.reply({'error': 'unmatched fixture route'}, 404)
    def do_DELETE(self):
        if urlparse(self.path).path == '/api/auth/web3/logout': return self.reply({'success': True})
        return self.reply({'error': 'unmatched fixture route'}, 404)

def browser(*args):
    result = subprocess.run(['agent-browser', '--session', 'epsx-dioxus-account-audit', *args], capture_output=True, text=True, timeout=35)
    if result.returncode: raise RuntimeError(f'Browser {args[0]}: {result.stderr} {result.stdout}')
    return result.stdout.strip()

def main():
    global JWKS, FAIL_PURCHASES, FAIL_UPLOAD, NOTIFICATION_TITLE
    OUT.mkdir(parents=True, exist_ok=True)
    private = rsa.generate_private_key(public_exponent=65537, key_size=2048)
    public = private.public_key().public_numbers()
    JWKS = {'keys': [{'kty': 'RSA', 'use': 'sig', 'alg': 'RS256', 'kid': 'ephemeral-account-audit', 'n': number(public.n), 'e': number(public.e)}]}
    upstream = ThreadingHTTPServer(('127.0.0.1', 0), Backend)
    threading.Thread(target=upstream.serve_forever, daemon=True).start()
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0)); port = sock.getsockname()[1]
    base = f'http://127.0.0.1:{port}'
    backend = f'http://127.0.0.1:{upstream.server_port}'
    now = int(time.time())
    header = b64(json.dumps({'alg': 'RS256', 'kid': 'ephemeral-account-audit', 'typ': 'JWT'}).encode())
    claims = b64(json.dumps({'iss': backend, 'sub': WALLET, 'aud': ['epsx-frontend'], 'exp': now + 600, 'iat': now - 1, 'jti': 'isolated-account-audit', 'scope': 'openid permissions', 'wallet_address': WALLET, 'auth_method': 'web3_siwe', 'auth_time': now - 1}).encode())
    signing = f'{header}.{claims}'
    token = signing + '.' + b64(private.sign(signing.encode(), padding.PKCS1v15(), hashes.SHA256()))
    env = dict(os.environ, EPSX_ENV='development', ENV='development', IP='127.0.0.1', HOST='127.0.0.1', PORT=str(port), API_URL=backend, BACKEND_URL=backend, CONTENT_SERVICE_URL=backend, NOTIFICATION_SERVICE_URL=backend, OIDC_ISSUER=backend, OIDC_JWKS_URL=f'{backend}/.well-known/jwks.json', DIOXUS_PUBLIC_PATH=str(BUNDLE / 'public'), EPSX_BROWSER_RUNTIME_DIR=str(ROOT / 'target/epsx-service-worker'))
    records = []
    with (OUT / 'server.log').open('w') as log:
        process = subprocess.Popen([str(BUNDLE / 'server')], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        try:
            for _ in range(100):
                if process.poll() is not None: raise RuntimeError('Fullstack server exited')
                try:
                    with urlopen(base + '/api/health', timeout=.3): break
                except OSError: time.sleep(.1)
            else: raise RuntimeError('Fullstack server did not start')
            if os.getenv('EPSX_AUDIT_OFFLINE') == '1':
                with urlopen(base+'/offline') as response:
                    guest=response.read()
                    assert response.headers['Cache-Control']=='public, max-age=300'
                with urlopen(Request(base+'/offline',headers={'Cookie':'epsx.frontend.access_token='+token})) as response:
                    owner=response.read()
                    assert response.headers['x-epsx-public-cache']=='offline-shell-v1'
                assert b'/public/enterprise.css?v=dioxus-2' in guest, 'offline SSR uses stale stylesheet version'
                assert guest==owner and WALLET.encode() not in owner
                assert COUNTERS['jwks']==0 and READS==[],(COUNTERS,READS)
                records.append({'offline_cookie_independent_ssr_no_auth_reads':True})
            browser('cookies', 'set', 'epsx.frontend.access_token', token, '--url', base, '--httpOnly', '--sameSite', 'Lax')
            browser('open', base + '/account/payments')
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('wait', '.fe-purchase-table')
            assert READS.count('/api/payments/pay-orders') == 1, READS
            browser('eval', 'window.originalDocument=document;')
            text = browser('get', 'text', '.fe-purchase-table')
            assert 'Paid' in text and 'Expired' in text and '5.00 USDT' in text, text
            browser('click', '.fe-purchase-actions button')
            browser('wait', '--fn', 'document.querySelector("[data-dioxus-purchases]").getAttribute("aria-busy") === "false"')
            assert browser('eval', 'document === originalDocument') == 'true'
            assert READS.count('/api/payments/pay-orders') == 2, READS
            browser('click', '.fe-purchase-plan')
            browser('wait', '.fe-purchase-details')
            assert browser('eval', 'document === originalDocument') == 'true'
            browser('back')
            browser('wait', '.fe-purchase-table')
            FAIL_PURCHASES = True
            browser('click', '.fe-purchase-actions button')
            browser('wait', '--text', 'Could not load results')
            assert 'Fixture day plan' in browser('get', 'text', '.fe-purchase-table')
            FAIL_PURCHASES = False
            browser('click', '[data-dioxus-purchases] > div[role="status"] button')
            browser('wait', '--fn', '!document.querySelector("[data-dioxus-purchases] > div[role=status]")')
            records.append({'purchases_ssr_single_read_refresh_detail_back_error_retry': True})
            for width, height, name in [(1440,900,'purchases-desktop'),(390,844,'purchases-mobile')]:
                browser('set','viewport',str(width),str(height))
                browser('screenshot',str(OUT / (name + '.png')))
                assert browser('eval', 'document.documentElement.scrollWidth <= innerWidth') == 'true'
            browser('set', 'viewport', '1440', '900')
            browser('open', base + '/account')
            browser('wait', '[data-dioxus-hydrated="true"]')
            browser('wait', '[data-dioxus-account]')
            browser('wait', '[data-preferences-form]')
            assert '12.5' in browser('get', 'text', '[data-dioxus-account]')
            browser('eval', 'window.originalDocument=document;')
            browser('select', '[data-preferences-form] select[name=email]', 'false')
            browser('click', '[data-preferences-form] button[type=submit]')
            browser('wait', '--text', 'Preferences saved.')
            assert WRITES[-1]['channels']['email'] is False, WRITES
            assert browser('eval', 'document === originalDocument') == 'true'
            browser('screenshot', str(OUT / 'account-desktop.png'))
            records.append({'account_owner_data_and_preferences_no_reload': True})
            if os.getenv('EPSX_AUDIT_DEVELOPER') == '1':
                browser('scrollintoview','a[href="/developer"]')
                browser('click','a[href="/developer"]')
                browser('wait','[data-developer-create-form]')
                assert browser('eval','document === originalDocument')=='true'
                browser('fill','[data-developer-create-form] input[name=name]','Fixture integration')
                browser('fill','[data-developer-create-form] textarea','Keep this description')
                browser('check','[data-developer-create-form] input[name=scopes]')
                browser('click','[data-developer-create]')
                browser('wait','--text','Could not load results. Please try again.')
                assert browser('get','value','[data-developer-create-form] input[name=name]')=='Fixture integration'
                assert browser('get','value','[data-developer-create-form] textarea')=='Keep this description'
                browser('click','[data-developer-create]')
                browser('wait','--text',DEVELOPER_SECRET)
                assert len(DEVELOPER_WRITES)==2 and DEVELOPER_WRITES[0][2]==DEVELOPER_WRITES[1][2] and DEVELOPER_WRITES[1][1]['description']=='Keep this description',DEVELOPER_WRITES
                browser('check','[data-developer-revoke-form] input[type=checkbox]')
                browser('click','[data-developer-revoke-form] button')
                browser('wait','--text','API key revoked.')
                assert len(DEVELOPER_WRITES)==3 and DEVELOPER_KEYS[0]['status']=='revoked',DEVELOPER_WRITES
                browser('scrollintoview','a[href="/developer/docs"]')
                browser('click','a[href="/developer/docs"]')
                browser('wait','[data-developer-docs-state="ready"]')
                assert browser('eval','document === originalDocument')=='true', 'Developer docs link reloaded document'
                browser('fill','[data-developer-docs-state] input[type=password]',DEVELOPER_SECRET)
                browser('scrollintoview','#operation-fixtureRead button.btn-primary')
                browser('click','#operation-fixtureRead button.btn-primary')
                browser('wait','--text','fixture_result')
                assert len(DEVELOPER_TRIES)==1
                assert browser('eval','document === originalDocument')=='true'
                browser('screenshot',str(OUT/'developer-docs-desktop.png'))
                records.append({'developer_shell_create_retry_preserves_draft_and_identity_revoke_try_no_reload':True})
            if os.getenv('EPSX_AUDIT_NOTIFICATIONS') == '1':
                browser('eval', "window.streamOpened=0;window.streamClosed=0;window.NativeEventSource=EventSource;window.EventSource=class extends NativeEventSource {constructor(...args){super(...args);streamOpened++;} close(){streamClosed++;super.close();}}")
                browser('click','a[aria-label="Notifications"]')
                browser('wait','[data-notification-id="fixture-notification"]')
                assert browser('eval','document === originalDocument')=='true'
                browser('click','button[data-notification-mutation="read"]')
                browser('wait','button[data-notification-mutation="unread"]')
                assert NOTIFICATION_READ
                browser('click','button[data-notification-mutation="unread"]')
                browser('wait','button[data-notification-mutation="read"]')
                assert not NOTIFICATION_READ
                NOTIFICATION_TITLE='Live fixture update'
                NOTIFICATION_EVENT.set()
                browser('wait','--text','Live fixture update')
                assert NOTIFICATION_STREAMS
                assert browser('eval','document === originalDocument')=='true'
                browser('screenshot',str(OUT/'notifications-desktop.png'))
                browser('scrollintoview','a[href="/account"]')
                browser('click','a[href="/account"]')
                browser('wait','[data-preferences-form]')
                browser('wait','--fn','streamClosed === 1')
                assert browser('eval','streamOpened === 1 && document === originalDocument')=='true'
                records.append({'notifications_shell_navigation_read_unread_named_sse_cleanup_no_reload':True})
            if os.getenv('EPSX_AUDIT_WATCHLIST') == '1':
                browser('open',base+'/portfolio')
                browser('wait','[data-dioxus-hydrated="true"]')
                browser('wait','[data-watchlist-organizer]')
                browser('eval','window.originalDocument=document;')
                browser('fill','#portfolio-watchlist-symbol','MSFT')
                browser('click','[data-watchlist-add]')
                browser('wait','--text','MSFT')
                assert WATCH['watched']==2,WATCH
                browser('click','[data-watchlist-new-group] > summary')
                browser('fill','#portfolio-new-group','Long term')
                browser('click','[data-watchlist-group-create]')
                browser('wait',f'[data-group-id="{GROUP}"]')
                browser('select','[data-symbol="AAPL"] select',GROUP)
                browser('wait','--fn',f'document.querySelector(`[data-group-id="{GROUP}"] [data-symbol="AAPL"]`) !== null')
                assert WATCH['groups'][0]['symbols']==['AAPL'],WATCH
                browser('click','[data-symbol="AAPL"] [data-watchlist-item-handle]')
                browser('press','Enter')
                browser('press','Tab')
                browser('press','Escape')
                assert WATCH['groups'][0]['symbols']==['AAPL'],WATCH
                browser('press','Enter')
                browser('press','Tab')
                browser('press','Enter')
                browser('wait','--fn','document.querySelector(`[data-group-id="ungrouped"] [data-symbol="AAPL"]`) !== null')
                assert WATCH['groups'][0]['symbols']==[] and 'AAPL' in WATCH['ungrouped'],WATCH
                assert browser('eval','document === originalDocument')=='true'
                browser('screenshot',str(OUT/'watchlist-desktop.png'))
                records.append({'watchlist_add_create_group_move_no_reload':True})
            if os.getenv('EPSX_AUDIT_CHAT') == '1':
                browser('open',base+'/chat?new=1')
                browser('wait','[data-dioxus-hydrated="true"]')
                browser('eval','window.originalDocument=document;')
                browser('click','.chat-topic-card')
                browser('fill','#chat-subject','Fixture support issue')
                browser('fill','#chat-message','Initial fixture message')
                fixture=OUT/'attachment.pdf'
                fixture.write_bytes(b'%PDF-1.4 fixture')
                browser('upload','input[type=file]',str(fixture))
                browser('wait','--text','attachment.pdf')
                browser('click','.chat-topic-start')
                browser('wait','--text','Retry sends only the attachment.')
                assert len(CHAT_CREATES)==1,CHAT_CREATES
                FAIL_UPLOAD=False
                browser('click','.chat-topic-start')
                browser('wait','--url',base+'/chat/'+CONVERSATION)
                browser('wait','.chat-input-textarea')
                assert len(CHAT_CREATES)==1 and len(UPLOADS)==2,(CHAT_CREATES,UPLOADS)
                browser('fill','.chat-input-textarea','Second fixture message')
                browser('click','.chat-input-send')
                browser('wait','--text','Second fixture message')
                assert browser('eval','document === originalDocument')=='true'
                browser('screenshot',str(OUT/'chat-desktop.png'))
                browser('open',base+'/chat/history')
                browser('wait','[data-dioxus-hydrated="true"]')
                browser('wait','--text','Fixture support issue')
                records.append({'chat_create_upload_retry_no_duplicate_send_history':True})
            if os.getenv('EPSX_AUDIT_OFFLINE') == '1':
                browser('open',base+'/offline')
                browser('wait','[data-dioxus-hydrated="true"]')
                browser('eval',"(async()=>{const old=await caches.open('epsx-public-recovery-v2'); await old.put('/offline',new Response('<link href=\"/public/enterprise.css?v=dioxus-1\">')); await caches.open('unrelated-fixture-cache'); return true;})()")
                browser('eval',"(async()=>{await navigator.serviceWorker.register('/runtime/epsx_service_worker_bootstrap.v3.js?rev=3',{type:'module',scope:'/'}); await Promise.race([navigator.serviceWorker.ready,new Promise((_,reject)=>setTimeout(()=>reject(new Error('worker installation timed out')),10000))]);return true;})()")
                browser('wait','--fn','navigator.serviceWorker.controller !== null')
                assert browser('eval',"(async()=>{const names=await caches.keys();const cache=await caches.open('epsx-public-recovery-v3');const html=await (await cache.match('/offline')).text();return !names.includes('epsx-public-recovery-v2') && names.includes('unrelated-fixture-cache') && html.includes('/public/enterprise.css?v=dioxus-2') && !!(await cache.match('/public/enterprise.css?v=dioxus-2'));})()")=='true'
                records.append({'offline_upgrade_replaces_v1_html_preserves_unrelated_cache':True})
                browser('open',base+'/account/payments')
                browser('wait','.fe-purchase-table')
                process.terminate()
                process.wait(timeout=5)
                browser('open',base+'/account/payments?offset=0')
                browser('wait','--text',"You're Offline")
                assert '/offline?return_url=' in browser('get','url')
                assert browser('eval',"getComputedStyle(document.querySelector('.fe-marketing-header')).display === 'flex'")=='true'
                browser('screenshot',str(OUT/'offline-cached.png'))
                process=subprocess.Popen([str(BUNDLE/'server')],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
                for _ in range(100):
                    try:
                        with urlopen(base+'/api/health',timeout=.3): break
                    except OSError: time.sleep(.1)
                browser('click','.offline-retry')
                browser('wait','--url',base+'/account/payments?offset=0')
                browser('wait','.fe-purchase-table')
                browser('wait','[data-dioxus-hydrated="true"]')
                records.append({'cached_offline_redirect_and_online_retry':True})
            browser('eval','window.logoutDocument=document;')
            browser('click', '.fe-wallet-trigger')
            browser('click', '.fe-wallet-disconnect')
            browser('wait','--fn',"location.pathname === '/'")
            assert browser('eval','document === logoutDocument')=='true'
            cookies = browser('cookies', 'get')
            assert 'epsx.frontend.access_token' not in cookies, 'logout did not clear access cookie'
            records.append({'logout_clears_session_cookie': True})
            (OUT / 'report.json').write_text(json.dumps(records, indent=2))
            print(json.dumps(records, indent=2))
        except Exception:
            print(browser('get','url'))
            print(browser('get','text','body')[:3000])
            print(browser('errors'))
            browser('screenshot',str(OUT/'failure.png'))
            raise
        finally:
            try: browser('set','offline','off')
            except Exception: pass
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired: process.kill()
            upstream.shutdown()
            browser('close')

if __name__ == '__main__': main()
