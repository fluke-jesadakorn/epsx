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
ROOT=shared.ROOT
BUNDLE=Path(os.environ.get('EPSX_AUDIT_FRONTEND_BUNDLE',str(ROOT/'target/dx/dx-frontend/debug/web')))
OUT=Path(os.environ.get('EPSX_AUTH_AUDIT_OUT',str(ROOT/'target/dioxus-migration/auth-payment')))
PLAN='11111111-1111-4111-8111-111111111111'
HASH='0x'+'a'*64
TOKEN=''
CONFIRMED=False
CALLS=[]
class Backend(shared.Backend):
 def do_GET(self):
  path=urlparse(self.path).path
  if path==f'/api/public/plans/{PLAN}':
   return self.reply({'success':True,'data':{'id':PLAN,'name':'Fixture day plan','plan_type':'day','current_price':'5.00','effective_price':5,'promotion_active':False,'promotion_status':'none','promotion_discount':0,'promotion_savings':'0','promotion_ends_at':None,'currency':'USD','billing_cycle':'one_time','features':['One day of research'],'permissions':[],'is_active':True,'tier_level':0,'plan_group':'personal','ranking_offset':0,'rankings_limit':10,'checkout_price':'5.00','settlement_currency':'USDT','duration_days':1}})
  if path.startswith('/api/payments/status/'):
   return self.reply({'success':True,'data':{'transaction_hash':HASH,'status':'confirmed' if CONFIRMED else 'pending','confirmations':3 if CONFIRMED else 0,'block_number':None,'error_message':None,'payment_reference':'fixture','plan_name':'Fixture day plan','amount':5,'currency':'USDT','completed_at':None,'last_checked_at':None}})
  return super().do_GET()
 def do_POST(self):
  path=urlparse(self.path).path
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))) or '{}')
  CALLS.append((path,body))
  if path=='/api/auth/web3/challenge':return self.reply({'success':True,'wallet_address':shared.WALLET,'nonce':'fixture-nonce','message':'Fixture sign-in verification','expires_at':int(time.time())+300})
  if path in ['/api/auth/web3/verify','/api/auth/session/refresh']:
   if path.endswith('/verify'):assert body['signature']=='0xfixture-signature' and body['client_id']=='epsx-frontend'
   return self.reply({'success':True,'authenticated':True,'wallet_address':shared.WALLET,'permissions':[],'access_token':TOKEN,'refresh_token':'fixture-refresh','expires_in':600,'refresh_expires_in':3600})
  if path=='/api/payments/submit':return self.reply({'success':True,'message':'Pending','data':{'payment_reference':'fixture','status':'pending','transaction_hash':HASH}})
  return self.reply({'error':'unmatched fixture route'},404)
def browser(*args):
 if args[0]=="screenshot" and os.environ.get("EPSX_SKIP_SCREENSHOTS")=="1":return "Skipped by explicit fixture option"
 result=subprocess.run(['agent-browser','--session',os.environ.get('EPSX_AUTH_BROWSER_SESSION','epsx-dioxus-auth-payment'),*args],capture_output=True,text=True,timeout=35)
 if result.returncode:raise RuntimeError(f'{args[0]}: {result.stderr} {result.stdout}')
 return result.stdout.strip()
def main():
 global TOKEN,CONFIRMED
 OUT.mkdir(parents=True,exist_ok=True)
 key=rsa.generate_private_key(public_exponent=65537,key_size=2048);public=key.public_key().public_numbers()
 shared.JWKS={'keys':[{'kty':'RSA','use':'sig','alg':'RS256','kid':'ephemeral-auth-payment','n':shared.number(public.n),'e':shared.number(public.e)}]}
 upstream=ThreadingHTTPServer(('127.0.0.1',0),Backend);threading.Thread(target=upstream.serve_forever,daemon=True).start()
 with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 base=f'http://127.0.0.1:{port}';backend=f'http://127.0.0.1:{upstream.server_port}';now=int(time.time())
 h=shared.b64(json.dumps({'alg':'RS256','kid':'ephemeral-auth-payment','typ':'JWT'}).encode());c=shared.b64(json.dumps({'iss':backend,'sub':shared.WALLET,'aud':['epsx-frontend'],'exp':now+600,'iat':now-1,'jti':'auth-payment','scope':'openid permissions','wallet_address':shared.WALLET,'auth_method':'web3_siwe','auth_time':now-1}).encode());sign=f'{h}.{c}';TOKEN=sign+'.'+shared.b64(key.sign(sign.encode(),padding.PKCS1v15(),hashes.SHA256()))
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),API_URL=backend,BACKEND_URL=backend,CONTENT_SERVICE_URL=backend,NOTIFICATION_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(BUNDLE/'public'),EPSX_PAY_CHECKOUT_ENABLED='false',NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL='0x'+'2'*40,NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL='0x'+'3'*40)
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
   if os.environ.get('EPSX_HOLD_AUTH_FIXTURE')=='1':
    print(json.dumps({'fixture_url':base+'/auth','server_pid':process.pid}),flush=True)
    while True:time.sleep(30)
   with urlopen(base+'/auth') as response:assert 'dist/tailwind.css' in response.read().decode()
   browser('open',base+f'/auth?return_url=%2Fpayment%2Fplan%2F{PLAN}')
   browser('wait','[data-dioxus-hydrated="true"]')
   assert browser('eval','!document.querySelector(".fe-sidebar")')=='true'
   browser('eval','delete window.ethereum;delete window.__epsxPayProvider')
   browser('click','.auth-card-cta button');browser('wait','--fn','document.querySelector("#auth-card-error")?.textContent.includes("Install or unlock MetaMask")')
   assert browser('eval','!document.querySelector(".auth-card-cta button").disabled')=='true'
   assert not [x for x in CALLS if x[0].endswith('/challenge')]
   records.append('No injected wallet produces a visible error and enabled retry without a challenge')
   for width,height in [(1440,900),(390,844)]:
    browser('set','viewport',str(width),str(height))
    for theme in ['light','dark']:
     is_dark=browser('eval','document.querySelector(".fe-auth").classList.contains("dark")')=='true'
     if is_dark!=(theme=='dark'):browser('click','.auth-page-theme-toggle button')
     browser('wait','200')
     assert browser('eval','parseFloat(getComputedStyle(document.querySelector(".auth-card")).paddingLeft)>=20')=='true'
     assert browser('eval','document.documentElement.scrollWidth<=innerWidth')=='true'
     browser('screenshot',str(OUT/f'auth-{width}-{theme}.png'))
   records.append('Auth card padding and desktop/mobile light/dark layouts verified')

   browser('eval',f'''window.originalDocument=document;window.rejectWallet=true;window.sendCount=0;window.ethereum={{request:async({{method}})=>{{if(method==='eth_requestAccounts'||method==='eth_accounts')return ['{shared.WALLET}'];if(method==='personal_sign'){{if(window.holdSign)await new Promise(resolve=>window.resolveSign=resolve);if(window.rejectWallet)throw Error('Fixture user rejected signing');return '0xfixture-signature';}}if(method==='eth_chainId')return '0x7a69';if(method==='eth_sendTransaction'){{window.sendCount++;return '{HASH}';}}if(method==='eth_getTransactionReceipt')return null;throw Error(method);}}}};''')
   dark=browser('eval','document.querySelector(".fe-auth").classList.contains("dark")')
   browser('click','.auth-page-theme-toggle button')
   browser('wait','--fn',f'document.querySelector(".fe-auth").classList.contains("dark") !== {dark}')
   records.append('Auth theme updates shared shell signal')
   browser('click','.auth-card-cta button');browser('wait','--text','Fixture user rejected signing')
   assert browser('eval','document===originalDocument')=='true'
   records.append('SIWE rejection renders recoverable Dioxus error')
   browser('eval','window.rejectWallet=false;window.holdSign=true');browser('click','.auth-card-cta button')
   browser('wait','--fn','typeof window.resolveSign==="function"')
   challenge_count=len([x for x in CALLS if x[0].endswith('/challenge')])
   browser('eval','document.querySelector(".auth-card-cta button").click();document.querySelector(".auth-card-cta button").click()')
   assert len([x for x in CALLS if x[0].endswith('/challenge')])==challenge_count
   browser('eval','window.resolveSign();window.holdSign=false')
   records.append('Repeated clicks during pending wallet signing do not duplicate authentication')
   browser('wait','--url',base+f'/payment/plan/{PLAN}');browser('wait','.payment-checkout')
   assert browser('eval','document===originalDocument')=='true'
   cookies=browser('cookies','get');assert 'epsx.frontend.access_token' in cookies and 'epsx.frontend.refresh_token' in cookies
   assert browser('eval','document.cookie.includes("access_token")')=='false'
   assert len([x for x in CALLS if x[0].endswith('/verify')])==1
   records.append('Typed SIWE verifies JWT, forwards both HttpOnly cookies and routes without reload')
   browser('click','.payment-checkout button[data-epsx-action="submit-plan-payment"]')
   browser('wait','--text','Waiting for verified confirmation')
   assert browser('eval','window.sendCount')=='1'
   browser('click','.payment-checkout button[data-epsx-action="submit-plan-payment"]')
   browser('wait','--text','Waiting for verified confirmation')
   assert browser('eval','window.sendCount')=='1'
   assert 'Payment confirmed' not in browser('get','text','#plan-payment-status')
   records.append('Prepared wallet transfer, duplicate hash reuse and pending confirmation')
   CONFIRMED=True;browser('wait','--text','Payment confirmed')
   assert browser('eval','document===originalDocument')=='true'
   records.append('Backend status alone completes payment')
   for width,height in [(1440,900),(390,844)]:
    browser('set','viewport',str(width),str(height));browser('screenshot',str(OUT/f'payment-{width}.png'))
    assert browser('eval','document.documentElement.scrollWidth<=innerWidth')=='true'
   assert browser('errors') in ['', '[]'], browser('errors')
   records.append('Desktop/mobile checkout no horizontal overflow')
   (OUT/'report.json').write_text(json.dumps(records,indent=2));print(json.dumps(records,indent=2))
  finally:
   process.terminate()
   try:process.wait(timeout=5)
   except subprocess.TimeoutExpired:process.kill()
   upstream.shutdown()
if __name__=='__main__':main()
