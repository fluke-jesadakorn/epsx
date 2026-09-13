"""Local typed media browser audit with ephemeral JWT and bounded fake storage.
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
OUT=ROOT/'target/dioxus-migration/admin-media'
PLAN='11111111-1111-4111-8111-111111111111'
HASH='0x'+'a'*64
TOKEN=''
CONFIRMED=False
CALLS=[]
READS=[]
WRITES=[]
ITEMS={'news':[{'key':'fixture.pdf','url':'https://invalid.example/fixture.pdf','size':12,'last_modified':None}],'public':[]}
META={'timestamp':'2026-09-10T00:00:00Z','version':'v1'}
class Backend(adminfixture.Backend):
 def do_GET(self):
  path=urlparse(self.path).path
  if path.startswith('/api/admin/media/'):
   assert self.headers.get('Authorization','').startswith('Bearer ')
   bucket=path.rsplit('/',1)[-1];READS.append(bucket)
   return self.reply({'success':True,'data':ITEMS[bucket],'meta':META})
  return super().do_GET()
 def do_DELETE(self):
  path=urlparse(self.path).path
  assert self.headers.get('Authorization','').startswith('Bearer ')
  WRITES.append({'path':path,'identity':self.headers.get('Idempotency-Key')})
  bucket='news' if '/news/' in path else 'public';key=path.rsplit('/',1)[-1]
  ITEMS[bucket]=[v for v in ITEMS[bucket] if v['key']!=key]
  return self.reply({'success':True,'data':{'bucket':bucket,'key':key,'deleted':True,'size':None},'meta':META})
 def do_POST(self):
  assert self.headers.get('Authorization','').startswith('Bearer ')
  body=self.rfile.read(int(self.headers.get('Content-Length','0')))
  assert self.path=='/api/admin/files/upload' and b'fixture upload' in body
  WRITES.append({'path':self.path,'identity':self.headers.get('Idempotency-Key')})
  ITEMS['public']=[{'key':'uploaded.pdf','url':'https://invalid.example/uploaded.pdf','size':14,'last_modified':None}]
  return self.reply({'success':True,'data':{'bucket':'public','key':'uploaded.pdf','deleted':False,'size':14},'meta':META})
def browser(*args):
 result=subprocess.run(['agent-browser','--session','epsx-dioxus-admin-media',*args],capture_output=True,text=True,timeout=35)
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
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),API_URL=backend,BACKEND_URL=backend,IDENTITY_SERVICE_URL=backend,CONTENT_SERVICE_URL=backend,NOTIFICATION_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(BUNDLE/'public'),EPSX_PAY_CHECKOUT_ENABLED='false',NEXT_PUBLIC_PAYMENT_RECEIVER_LOCAL='0x'+'2'*40,NEXT_PUBLIC_PAYMENT_TOKEN_LOCAL='0x'+'3'*40)
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
   browser('open',base+'/media');browser('wait','--text','fixture.pdf');time.sleep(1)
   assert READS==['news'],READS
   browser('eval','window.originalDocument=document')
   browser('find','role','button','click','--name','Delete object');browser('wait','--text','Media object deleted')
   assert len(WRITES)==1 and WRITES[0]['identity']
   browser('find','role','link','click','--name','Public','--exact');browser('wait','--url','**bucket=public')
   browser('wait','--text','No files in “public”')
   assert browser('eval','document===originalDocument')=='true'
   records.append('One SSR read; authenticated delete and bucket Router transition without reload')
   sample=OUT/'fixture.pdf';sample.write_text('fixture upload')
   browser('click','details summary')
   browser('upload','input[type=file]',str(sample));browser('find','role','button','click','--name','Upload file')
   browser('wait','--text','Media object uploaded');browser('wait','--text','uploaded.pdf')
   assert len(WRITES)==2 and WRITES[1]['identity']!=WRITES[0]['identity']
   assert browser('eval','document===originalDocument')=='true'
   records.append('File adapter and typed server mutation upload once; inventory refresh without reload')
   browser('back');browser('wait','--url',base+'/media');time.sleep(.4)
   assert READS[-1]=='news'
   records.append('Back restores News bucket with matching data')
   for width,height in [(1440,900),(390,844)]:
    browser('set','viewport',str(width),str(height));browser('screenshot',str(OUT/f'media-{width}.png'))
    assert browser('eval','document.documentElement.scrollWidth<=innerWidth')=='true'
   records.append('Media desktop/mobile no horizontal overflow')
   (OUT/'report.json').write_text(json.dumps(records,indent=2));print(json.dumps(records,indent=2))
  finally:
   process.terminate()
   try:process.wait(timeout=5)
   except subprocess.TimeoutExpired:process.kill()
   upstream.shutdown()
if __name__=='__main__':main()
