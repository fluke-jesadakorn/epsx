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
BUNDLE=ROOT/'target/dx/epsx-pay/debug/web'
OUT=ROOT/'target/dioxus-migration/pay-auth'
PLAN='11111111-1111-4111-8111-111111111111'
HASH='0x'+'a'*64
TOKEN=''
CONFIRMED=False
CALLS=[]
class Backend(adminfixture.Backend):
 def do_GET(self):
  path=urlparse(self.path).path
  if path=='/api/v1/pay/config':return self.reply({'environments':[{'environment':'test','tokens':{'USDT':{'decimals':6}}}]})
  if path=='/api/v1/pay/merchants/me':
   if not self.headers.get('Authorization','').startswith('Bearer '):return self.reply({},401)
   return self.reply({'merchant_id':'m_fixture','name':'Typed auth fixture','wallet':shared.WALLET})
  if path.startswith('/api/v1/pay/'):return self.reply({'items':[]})
  if path==f'/api/public/plans/{PLAN}':
   return self.reply({'success':True,'data':{'id':PLAN,'name':'Fixture day plan','plan_type':'day','current_price':'5.00','effective_price':5,'promotion_active':False,'promotion_status':'none','promotion_discount':0,'promotion_savings':'0','promotion_ends_at':None,'currency':'USD','billing_cycle':'one_time','features':['One day of research'],'permissions':[],'is_active':True,'tier_level':0,'plan_group':'personal','ranking_offset':0,'rankings_limit':10,'checkout_price':'5.00','settlement_currency':'USDT','duration_days':1}})
  if path.startswith('/api/payments/status/'):
   return self.reply({'success':True,'data':{'transaction_hash':HASH,'status':'confirmed' if CONFIRMED else 'pending','confirmations':3 if CONFIRMED else 0,'block_number':None,'error_message':None,'payment_reference':'fixture','plan_name':'Fixture day plan','amount':5,'currency':'USDT','completed_at':None,'last_checked_at':None}})
  return super().do_GET()
 def do_DELETE(self):
  return self.reply({'success':True})
 def do_POST(self):
  path=urlparse(self.path).path
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))) or '{}')
  CALLS.append((path,body))
  if path=='/api/auth/web3/challenge':return self.reply({'success':True,'wallet_address':shared.WALLET,'nonce':'fixture-nonce','message':'Fixture sign-in verification','expires_at':int(time.time())+300})
  if path in ['/api/auth/web3/verify','/api/auth/session/refresh']:
   if path.endswith('/verify'):assert body['signature']=='0xfixture-signature' and body['client_id']=='epsx-pay'
   return self.reply({'success':True,'authenticated':True,'wallet_address':shared.WALLET,'permissions':[],'access_token':TOKEN,'refresh_token':'fixture-refresh','expires_in':600,'refresh_expires_in':3600})
  if path=='/api/payments/submit':return self.reply({'success':True,'message':'Pending','data':{'payment_reference':'fixture','status':'pending','transaction_hash':HASH}})
  return self.reply({'error':'unmatched fixture route'},404)
def browser(*args):
 result=subprocess.run(['agent-browser','--session','epsx-dioxus-pay-auth',*args],capture_output=True,text=True,timeout=35)
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
 h=shared.b64(json.dumps({'alg':'RS256','kid':'ephemeral-auth-payment','typ':'JWT'}).encode());c=shared.b64(json.dumps({'iss':backend,'sub':shared.WALLET,'aud':['epsx-pay'],'exp':now+600,'iat':now-1,'jti':'auth-payment','scope':'openid permissions','wallet_address':shared.WALLET,'auth_method':'web3_siwe','auth_time':now-1}).encode());sign=f'{h}.{c}';TOKEN=sign+'.'+shared.b64(key.sign(sign.encode(),padding.PKCS1v15(),hashes.SHA256()))
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),API_URL=backend,BACKEND_URL=backend,PAYMENT_SERVICE_URL=backend,PAY_FRONTEND_URL=base,IDENTITY_SERVICE_URL=backend,CONTENT_SERVICE_URL=backend,NOTIFICATION_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(BUNDLE/'public'),EPSX_PAY_CHECKOUT_ENABLED='false',NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL='0x'+'2'*40,NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL='0x'+'3'*40)
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
   browser('cookies','clear')
   browser('open',base+'/dashboard');browser('wait','--text','Connect MetaMask');time.sleep(.5)
   browser('eval',f'''window.originalDocument=document;window.rejectWallet=true;window.ethereum={{request:async({{method}})=>{{if(method==='eth_requestAccounts'||method==='eth_accounts')return ['{shared.WALLET}'];if(method==='personal_sign'){{if(window.rejectWallet)throw Error('Fixture rejected Pay signing');return '0xfixture-signature';}}throw Error(method);}}}};''')
   browser('find','role','button','click','--name','Connect MetaMask');browser('wait','--text','Fixture rejected Pay signing')
   assert browser('eval','document===originalDocument')=='true'
   browser('eval','window.rejectWallet=false');browser('find','role','button','click','--name','Connect MetaMask')
   browser('wait','--text','Typed auth fixture')
   assert browser('eval','document===originalDocument')=='true'
   cookies=browser('cookies','get');assert 'epsx.pay.access_token' in cookies and 'epsx.pay.refresh_token' in cookies
   assert 'epsx.frontend.access_token' not in cookies and 'epsx.admin.access_token' not in cookies
   assert browser('eval','document.cookie.includes("access_token")')=='false'
   records.append('Pay typed SIWE verifies Pay JWT and sets both HttpOnly Pay-only cookies')
   browser('find','role','button','click','--name','Sign out');browser('wait','--text','Connect MetaMask')
   assert 'epsx.pay.access_token' not in browser('cookies','get')
   assert browser('eval','document===originalDocument')=='true'
   records.append('Pay typed logout clears session and updates merchant UI without reload')
   (OUT/'report.json').write_text(json.dumps(records,indent=2));print(json.dumps(records,indent=2))
  finally:
   process.terminate()
   try:process.wait(timeout=5)
   except subprocess.TimeoutExpired:process.kill()
   upstream.shutdown()
if __name__=='__main__':main()
