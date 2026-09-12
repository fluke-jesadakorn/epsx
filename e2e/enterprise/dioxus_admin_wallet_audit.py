"""Local Dioxus SIWE/checkout browser audit with ephemeral JWT and fake wallet.
No real signatures, payments, persisted keys or production services are used.
"""
import json, os, socket, subprocess, threading, time, shutil
from pathlib import Path
from http.server import ThreadingHTTPServer
from urllib.parse import urlparse
from urllib.request import urlopen
from cryptography.hazmat.primitives.asymmetric import rsa,padding
from cryptography.hazmat.primitives import hashes
import dioxus_account_audit as shared
import importlib.util
spec=importlib.util.spec_from_file_location("adminfixture",shared.ROOT/"apps/admin/fixtures/fullstack_browser.py")
adminfixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(adminfixture)
ROOT=shared.ROOT
BUNDLE=ROOT/'target/dx/dx-admin/debug/web'
OUT=ROOT/'target/dioxus-migration/admin-wallet'
PLAN='11111111-1111-4111-8111-111111111111'
HASH='0x'+'a'*64
TOKEN=''
CONFIRMED=False
CALLS=[]
WALLET={'address':shared.WALLET,'chain_id':'56','label':'Fixture wallet','role':'user','status':'active','metadata':{},'version':1,'created_at':None}
SUBPLAN={'id':PLAN,'merchant_id':PLAN,'name':'Fixture subscription','description':'Research access','amount':'500','currency':'USDT','chain_id':'56','interval':30,'active':False,'created_at':None,'version':2}
class Backend(adminfixture.Backend):
 def do_GET(self):
  parsed=urlparse(self.path);path=parsed.path
  if path=='/api/v1/admin/wallets/stats':return self.reply({'total':60,'active':60,'disabled':0,'new_30_days':1,'correlation_id':'fixture'})
  if path=='/api/v1/admin/wallets':
   from urllib.parse import parse_qs
   q=parse_qs(parsed.query);limit=int(q.get('limit',['10'])[0]);offset=int(q.get('offset',['0'])[0])
   return self.reply({'items':[WALLET],'total':60,'limit':limit,'offset':offset,'correlation_id':'fixture'})
  if path=='/api/v1/admin/wallets/'+shared.WALLET:return self.reply(WALLET)
  if path=='/api/v1/admin/subscription/access':return self.reply({'items':[],'correlation_id':'fixture'})
  if path=='/api/v1/admin/credits':return self.reply({'outstanding_minor':500,'granted_today_minor':0,'revoked_today_minor':0,'active_accounts':1,'correlation_id':'fixture'})
  if path=='/api/v1/admin/subscription/plans':return self.reply({'items':[SUBPLAN],'total':1,'limit':100,'offset':0,'correlation_id':'fixture'})
  if path=='/api/v1/admin/subscription/plans/'+PLAN:return self.reply(SUBPLAN)
  return super().do_GET()
 def do_POST(self):
  path=urlparse(self.path).path;body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))) or '{}')
  CALLS.append((path,body,self.headers.get('Idempotency-Key')))
  if body.get('reason')=='Fixture conflict':return self.reply({'error':'conflict'},409)
  if path.endswith('/disable'):
   WALLET['status']='disabled';WALLET['version']=2
   return self.reply({'wallet':WALLET,'evidence':{'operation_id':PLAN,'version':2,'observed_at':'2026-09-10T00:00:00Z'},'correlation_id':'fixture'})
  return self.reply({'success':True})
 def do_PUT(self):return self.do_POST()
def browser(*args):
 result=subprocess.run(['agent-browser','--session','epsx-dioxus-admin-wallet',*args],capture_output=True,text=True,timeout=35)
 if result.returncode:raise RuntimeError(f'{args[0]}: {result.stderr} {result.stdout}')
 return result.stdout.strip()
def main():
 global TOKEN,CONFIRMED
 OUT.mkdir(parents=True,exist_ok=True)
 key=rsa.generate_private_key(public_exponent=65537,key_size=2048);public=key.public_key().public_numbers()
 shared.JWKS={'keys':[{'kty':'RSA','use':'sig','alg':'RS256','kid':'ephemeral-auth-payment','n':shared.number(public.n),'e':shared.number(public.e)}]}
 adminfixture.JWKS=shared.JWKS
 upstream=ThreadingHTTPServer(('127.0.0.1',0),Backend);threading.Thread(target=upstream.serve_forever,daemon=True).start()
 with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 base=f'http://127.0.0.1:{port}';backend=f'http://127.0.0.1:{upstream.server_port}';now=int(time.time())
 h=shared.b64(json.dumps({'alg':'RS256','kid':'ephemeral-auth-payment','typ':'JWT'}).encode());c=shared.b64(json.dumps({'iss':backend,'sub':shared.WALLET,'aud':['epsx-admin'],'exp':now+600,'iat':now-1,'jti':'auth-payment','scope':'openid permissions','wallet_address':shared.WALLET,'auth_method':'web3_siwe','auth_time':now-1}).encode());sign=f'{h}.{c}';TOKEN=sign+'.'+shared.b64(key.sign(sign.encode(),padding.PKCS1v15(),hashes.SHA256()))
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),API_URL=backend,BACKEND_URL=backend,IDENTITY_SERVICE_URL=backend,CONTENT_SERVICE_URL=backend,NOTIFICATION_SERVICE_URL=backend,WALLET_SERVICE_URL=backend,SUBSCRIPTION_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(BUNDLE/'public'),EPSX_PAY_CHECKOUT_ENABLED='false',NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL='0x'+'2'*40,NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL='0x'+'3'*40)
 records=[]
 with (OUT/'server.log').open('w') as log:
  binary=OUT/f'server-{os.getpid()}';shutil.copy2(BUNDLE/'server',binary)
  process=subprocess.Popen([str(binary)],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  try:
   for _ in range(100):
    if process.poll() is not None:raise RuntimeError('server exited')
    try:
     with urlopen(base+'/api/health',timeout=.3):break
    except OSError:time.sleep(.1)
   browser('cookies','set','epsx.admin.access_token',TOKEN,'--url',base,'--httpOnly','--sameSite','Lax')
   browser('open',base+'/wallet-management/wallets?limit=25&status=active');browser('wait','--text','Fixture wallet');browser('wait','2000')
   browser('eval','window.originalDocument=document');assert browser('get','value','select[aria-label="Wallets per page"]')=='25'
   browser('select','select[aria-label="Wallets per page"]','50');browser('wait','--url','**limit=50')
   assert browser('eval','document===originalDocument')=='true'
   browser('find','role','button','click','--name','Next');browser('wait','--text','Page 2')
   browser('back');browser('wait','--text','Page 1')
   records.append('Wallet page size and pagination preserve filters, URL history and document')
   browser('find','role','link','click','--name','Fixture wallet');browser('wait','--text','Wallet details')
   browser('find','role','link','click','--name','Disable wallet');browser('wait','--text','Confirm disable')
   browser('fill','input[name="reason"]','Fixture administrative disable');browser('find','role','button','click','--name','Confirm disable');browser('wait','--text','Saved.')
   assert CALLS[-1][0].endswith('/disable') and CALLS[-1][1]['expected_version']==1 and CALLS[-1][2]
   records.append('Versioned wallet disable uses one scoped command and backend evidence')
   browser('eval',"document.querySelector('main a[href=\"/wallet-management/access\"]').click()")
   browser('wait','--text','Assign access')
   for field,value in [('wallet_address',shared.WALLET),('plan_id',PLAN),('permission','epsx:analytics:read'),('expected_version','0')]:browser('fill',f'input[name="{field}"]',value)
   browser('find','role','button','click','--name','Assign access');browser('wait','--text','Saved.')
   assert CALLS[-1][0].endswith('/assign') and CALLS[-1][1]['expected_version']==0
   records.append('Access assignment preserves backend permission and version validation')
   browser('eval',"document.querySelector('main a[href=\"/wallet-management/credits\"]').click()")
   browser('wait','--text','Grant credits')
   for field,value in [('wallet_address',shared.WALLET),('amount_minor','100'),('expected_version','1'),('reason','Fixture conflict')]:browser('fill',f'main form:nth-of-type(1) input[name="{field}"]',value)
   browser('find','role','button','click','--name','Grant credits');browser('wait','--text','This record changed.')
   assert browser('get','value','main form:nth-of-type(1) input[name="amount_minor"]')=='100'
   browser('fill','main form:nth-of-type(1) input[name="reason"]','Fixture grant')
   browser('find','role','button','click','--name','Grant credits');browser('wait','--text','Saved.')
   assert CALLS[-1][0].endswith('/grant') and CALLS[-1][1]['amount_minor']==100
   records.append('Credit form reaches verified backend with version and idempotency key')
   browser('eval',"document.querySelector('main a[href=\"/wallet-management/access/plans\"]').click()")
   browser('wait','--text','Fixture subscription');browser('find','role','link','click','--name','Fixture subscription · 500 USDT')
   browser('wait','--text','Edit subscription plan');assert browser('get','value','select[name="active"]')=='false'
   browser('fill','input[name="name"]','Updated subscription');browser('find','role','button','click','--name','Edit subscription plan');browser('wait','--text','Saved.')
   assert CALLS[-1][1]['expected_version']==2 and CALLS[-1][1]['active']==False
   assert browser('eval','document===originalDocument')=='true'
   records.append('Subscription edit preserves inactive state and backend version without reload')
   browser('eval',"document.querySelector('a[href=\"/wallet-management/credits\"]').click()")
   browser('wait','--text','Grant credits');assert browser('eval','document===originalDocument')=='true'
   records.append('Admin sidebar wallet child navigation preserves the hydrated document')
   for width,height in [(1440,900),(390,844)]:
    browser('set','viewport',str(width),str(height));time.sleep(.4);browser('screenshot',str(OUT/f'wallets-{width}.png'))
    assert browser('eval','document.documentElement.scrollWidth<=innerWidth')=='true'
   records.append('Wallet desktop/mobile layout has no horizontal overflow')
   (OUT/'report.json').write_text(json.dumps(records,indent=2));print(json.dumps(records,indent=2))
  finally:
   (OUT/'partial-report.json').write_text(json.dumps({'checks':records,'calls':CALLS},indent=2))
   process.terminate()
   try:process.wait(timeout=5)
   except subprocess.TimeoutExpired:process.kill()
   upstream.shutdown()
if __name__=='__main__':main()
