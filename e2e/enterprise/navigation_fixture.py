"""Run the navigation crawl with isolated upstreams and ephemeral sessions."""
import json, os, shutil, socket, subprocess, sys, threading, time
from pathlib import Path
from http.server import ThreadingHTTPServer
from urllib.request import urlopen
from urllib.parse import urlparse, parse_qs
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.primitives import hashes
import dioxus_account_audit as account
import dioxus_analytics_session_audit as frontend
import dioxus_admin_wallet_audit as admin
ROOT=Path(__file__).resolve().parents[2]
class AdminBackend(admin.Backend):
 def do_GET(self):
  path=urlparse(self.path).path;q=parse_qs(urlparse(self.path).query)
  wallet={'wallet_address':account.WALLET,'is_active':True,'created_at':'2026-01-01T00:00:00Z','last_auth_at':None}
  if path=='/api/admin/wallets':
   page=int(q.get('page',['1'])[0]);limit=int(q.get('limit',['10'])[0])
   return self.reply({'success':True,'data':{'wallets':[wallet],'total':60,'pagination':{'page':page,'limit':limit,'has_next':page*limit<60,'has_prev':page>1}}})
  if path=='/api/admin/wallets/'+account.WALLET:return self.reply({'success':True,'data':{**wallet,'permissions':[]}})
  if path=='/api/permissions/assignments':return self.reply({'success':True,'data':[]})
  if path=='/api/permissions/plans':return self.reply({'success':True,'data':[{'id':admin.PLAN,'name':'Fixture access plan'}]})
  if path=='/api/payments/admin/credits/stats':return self.reply({'success':True,'data':{'total_credits_outstanding':'100','total_credits_granted_today':'10','total_credits_used_today':'5','active_users_with_credits':1,'total_transactions_today':1,'average_balance':'100'}})
  return super().do_GET()
def main():
 app=sys.argv[1];mode=sys.argv[2] if len(sys.argv)>2 else 'authenticated'
 fixture=frontend if app=='frontend' else admin
 bundle=ROOT/f'target/dx/dx-{app}/debug/web'
 out=ROOT/'target/navigation-audit'/f'{app}-{mode}';out.mkdir(parents=True,exist_ok=True)
 key=rsa.generate_private_key(public_exponent=65537,key_size=2048);public=key.public_key().public_numbers()
 jwks={'keys':[{'kty':'RSA','use':'sig','alg':'RS256','kid':'navigation-fixture','n':account.number(public.n),'e':account.number(public.e)}]}
 account.JWKS=jwks;admin.adminfixture.JWKS=jwks
 upstream=ThreadingHTTPServer(('127.0.0.1',0),fixture.Backend if app=='frontend' else AdminBackend);threading.Thread(target=upstream.serve_forever,daemon=True).start()
 with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
 base=f'http://127.0.0.1:{port}';backend=f'http://127.0.0.1:{upstream.server_port}';now=int(time.time())
 header=account.b64(json.dumps({'alg':'RS256','kid':'navigation-fixture'}).encode())
 claims=account.b64(json.dumps({'iss':backend,'sub':account.WALLET,'aud':[f'epsx-{app}'],'exp':now+3600,'iat':now-1,'jti':'navigation-fixture','scope':'openid permissions','wallet_address':account.WALLET,'auth_method':'web3_siwe','auth_time':now-1,'permissions':['admin:*:*'] if app=='admin' else []}).encode())
 unsigned=f'{header}.{claims}';token=unsigned+'.'+account.b64(key.sign(unsigned.encode(),padding.PKCS1v15(),hashes.SHA256()));fixture.TOKEN=token
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),FRONTEND_URL=base,ADMIN_URL=base,API_URL=backend,BACKEND_URL=backend,IDENTITY_SERVICE_URL=backend,CONTENT_SERVICE_URL=backend,NOTIFICATION_SERVICE_URL=backend,WALLET_SERVICE_URL=backend,SUBSCRIPTION_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(bundle/'public'),EPSX_PAY_CHECKOUT_ENABLED='false')
 with (out/'server.log').open('w') as log:
  binary=out/f'server-{os.getpid()}';shutil.copy2(bundle/'server',binary)
  process=subprocess.Popen([str(binary)],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  try:
   for _ in range(200):
    if process.poll() is not None:raise RuntimeError('Fixture exited; inspect server.log')
    try:
     with urlopen(base+'/api/health',timeout=.3):break
    except OSError:time.sleep(.1)
   else:raise RuntimeError('Fixture not ready')
   child=dict(os.environ,EPSX_NAV_TOKEN=token if mode=='authenticated' else '',EPSX_NAV_OUT=str(out))
   subprocess.run([sys.executable,str(ROOT/'e2e/enterprise/dioxus_navigation_audit.py'),app,base],env=child,check=True)
  finally:
   process.terminate();process.wait(timeout=10);upstream.shutdown();binary.unlink(missing_ok=True)
if __name__=='__main__':main()
