"""Local typed news browser audit with ephemeral JWT and bounded fake storage.
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
OUT=ROOT/'target/dioxus-migration/admin-news'
PLAN='11111111-1111-4111-8111-111111111111'
HASH='0x'+'a'*64
TOKEN=''
CONFIRMED=False
CALLS=[]
READS=[]
WRITES=[]
CONFLICT=False
ARTICLE={'id':PLAN,'title':'Fixture article','slug':'fixture-article','summary':'Fixture summary','content':'# Fixture content','cover_image_url':None,'author_wallet':shared.WALLET,'status':'draft','tags':[],'published_at':None,'created_at':'2026-09-10T00:00:00Z','updated_at':'2026-09-10T00:00:00Z','is_pinned':False,'pinned_at':None}
ARTICLES={PLAN:ARTICLE}
META={'timestamp':'2026-09-10T00:00:00Z','version':'v1'}
class Backend(adminfixture.Backend):
 def do_GET(self):
  path=urlparse(self.path).path
  if path.startswith('/api/admin/news'):
   assert self.headers.get('Authorization','').startswith('Bearer ')
   READS.append(self.path)
   if path=='/api/admin/news':
    from urllib.parse import parse_qs
    query=parse_qs(urlparse(self.path).query);page=int(query.get('page',['1'])[0]);status=query.get('status',['all'])[0]
    rows=[v for v in ARTICLES.values() if status=='all' or v['status']==status]
    return self.reply({'success':True,'data':{'articles':rows,'total':len(rows),'page':page,'limit':20},'meta':META})
   return self.reply({'success':True,'data':ARTICLES[path.rsplit('/',1)[-1]],'meta':META})
  return super().do_GET()
 def do_DELETE(self):
  assert self.headers.get('Authorization','').startswith('Bearer ')
  item=self.path.rsplit('/',1)[-1];WRITES.append({'path':self.path,'identity':self.headers.get('Idempotency-Key')});ARTICLES.pop(item)
  return self.reply({'success':True,'data':{'id':item,'deleted':True},'meta':META})
 def do_POST(self):return self.do_PUT()
 def do_PUT(self):
  global CONFLICT
  assert self.headers.get('Authorization','').startswith('Bearer ')
  raw=self.rfile.read(int(self.headers.get('Content-Length','0')))
  WRITES.append({'path':self.path,'identity':self.headers.get('Idempotency-Key'),'version':self.headers.get('If-Match')})
  if self.path.endswith('/upload-image'):
   return self.reply({'success':True,'data':{'url':'https://images.example/cover.png','thumb_url':None,'filename':'fixture.png','mime':'image/png','size':14},'meta':META})
  if CONFLICT:CONFLICT=False;return self.reply({'success':False},409)
  body=json.loads(raw or '{}')
  if self.path=='/api/admin/news':
   item=dict(ARTICLE);item.update(body);item['id']='22222222-2222-4222-8222-222222222222';item['slug']='created-article';ARTICLES[item['id']]=item
  else:
   suffix=self.path.rsplit('/',1)[-1];item=ARTICLES[PLAN]
   if suffix in ['publish','unpublish']:item['status']='published' if suffix=='publish' else 'draft';item['published_at']='2026-09-10T00:00:00Z' if suffix=='publish' else None
   elif suffix in ['pin','unpin']:item['is_pinned']=suffix=='pin';item['pinned_at']='2026-09-10T00:00:00Z' if suffix=='pin' else None
   else:item.update({k:v for k,v in body.items() if v is not None})
  return self.reply({'success':True,'data':item,'meta':META})
def browser(*args):
 result=subprocess.run(['agent-browser','--session','epsx-dioxus-admin-news',*args],capture_output=True,text=True,timeout=35)
 if result.returncode:raise RuntimeError(f'{args[0]}: {result.stderr} {result.stdout}')
 return result.stdout.strip()
def main():
 global TOKEN,CONFIRMED,CONFLICT
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
   browser('open',base+'/news');browser('wait','--text','Fixture article');time.sleep(1)
   assert READS==['/api/admin/news?page=1&limit=20'],READS
   browser('eval','window.originalDocument=document')
   browser('click','a[aria-label="Edit article"]');browser('wait','textarea[name=content]')
   assert browser('get','value','textarea[name=content]')=='# Fixture content'
   browser('fill','input[name=title]','Updated fixture article')
   CONFLICT=True;browser('click','button[data-admin-news-submit]');browser('wait','--text','The article changed.')
   assert browser('get','value','input[name=title]')=='Updated fixture article'
   browser('click','button[data-admin-news-submit]');browser('wait','--text','Article saved.')
   assert WRITES[0]['identity']==WRITES[1]['identity'] and WRITES[1]['version']
   assert ARTICLE['title']=='Updated fixture article'
   assert browser('eval','document===originalDocument')=='true'
   records.append('SSR read once; typed update preserves fields after conflict, retries identity/version, no document reload')
   browser('click','button[data-admin-news-transition="publish"]');browser('wait','--text','Article saved.')
   time.sleep(.5);assert ARTICLE['status']=='published'
   records.append('Publish is an authenticated versioned backend command')
   browser('fill','input[name=title]','Unsaved title retained')
   sample=OUT/'fixture.png';sample.write_text('fixture upload')
   browser('upload','input[type=file]',str(sample));browser('find','role','button','click','--name','Upload image')
   browser('wait','--text','Image uploaded.')
   assert browser('get','value','input[name=title]')=='Unsaved title retained'
   assert browser('get','value','input[name=cover_image_url]')=='https://images.example/cover.png'
   browser('click','button[data-admin-news-submit]');browser('wait','--text','Article saved.')
   time.sleep(.3);assert ARTICLE['cover_image_url']=='https://images.example/cover.png'
   records.append('Typed image upload preserves unsaved text; save commits returned cover URL')
   browser('find','role','link','click','--name','Cancel');browser('wait','--url',base+'/news')
   browser('wait','a[href="/news/create"]');browser('click','a[href="/news/create"]');browser('wait','textarea[name=content]')
   browser('fill','input[name=title]','New fixture article');browser('fill','textarea[name=content]','# New content')
   browser('click','button[data-admin-news-submit]');browser('wait','--url','**/news/22222222-2222-4222-8222-222222222222/edit')
   browser('wait','[data-admin-news-editor-state="ready"]')
   assert browser('get','value','input[name=title]')=='New fixture article'
   assert len(ARTICLES)==2
   assert browser('eval','document===originalDocument')=='true'
   records.append('Create returns committed article and Router replaces editor URL without reload')
   for width,height in [(1440,900),(390,844)]:
    browser('set','viewport',str(width),str(height));time.sleep(.4);browser('screenshot',str(OUT/f'news-{width}.png'))
    assert browser('eval','document.documentElement.scrollWidth<=innerWidth')=='true'
   records.append('News editor desktop/mobile no horizontal overflow')
   (OUT/'report.json').write_text(json.dumps(records,indent=2));print(json.dumps(records,indent=2))
  finally:
   process.terminate()
   try:process.wait(timeout=5)
   except subprocess.TimeoutExpired:process.kill()
   upstream.shutdown()
if __name__=='__main__':main()
