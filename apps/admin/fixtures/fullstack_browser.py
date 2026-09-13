"""Local signed-session Fullstack fixture; never contacts production.
Run after dx build. Open the printed login URL using a browser test client.
"""
import base64, json, os, socket, subprocess, threading, time, shutil
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlparse
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.primitives import hashes
ROOT=Path(__file__).resolve().parents[3]
BUNDLE=ROOT/'target/dx/dx-admin/debug/web'
OUT=ROOT/'target/dioxus-migration/admin';OUT.mkdir(parents=True,exist_ok=True)
PLAN={'id':'11111111-1111-4111-8111-111111111111','name':'Fixture day plan','description':'Local test only','is_active':True,'billing_model':'subscription','permissions':['epsx:analytics:read'],'metadata':{'duration_days':1,'pay_prices':{'USDT':'5.00','USDC':'5.00'},'promotion':None}}
SETTINGS={'general':{'systemName':'Fixture EPSX','maintenanceMode':False}}
CONVERSATION={'id':PLAN['id'],'topic_id':PLAN['id'],'wallet_address':'0x1111111111111111111111111111111111111111','subject':'Fixture support request','status':'open','assigned_agent':None,'last_message_at':'2026-09-09T03:37:17Z','unread_user':0,'unread_agent':0,'created_at':'2026-09-09T03:37:17Z','updated_at':'2026-09-09T03:37:17Z'}
MESSAGES=[]
API_KEYS=[]
READS=[]; WRITES=[]; TOKEN=''; BASE=''; JWKS={}
class Backend(BaseHTTPRequestHandler):
 def log_message(self,*_):pass
 def reply(self,obj,status=200):
  if self.path.startswith("/api/admin/developer-portal") and isinstance(obj,dict) and obj.get("success"):obj["meta"]={"timestamp":"2026-09-09T03:37:17Z","request_id":"11111111-1111-4111-8111-111111111111","version":"v1"}
  self.send_response(status);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(json.dumps(obj).encode())
 def do_GET(self):
  path=urlparse(self.path).path
  if path=='/__fixture/login':
   self.send_response(303);self.send_header('Set-Cookie',f'epsx.admin.access_token={TOKEN}; Path=/; HttpOnly; SameSite=Lax');self.send_header('Location',BASE+'/plans');self.end_headers();return
  if path=='/__fixture/report':return self.reply({'reads':READS,'writes':WRITES,'plan':PLAN,'settings':SETTINGS})
  if path.endswith('/jwks.json'):return self.reply(JWKS)
  if not self.headers.get('Authorization','').startswith('Bearer '):return self.reply({},401)
  READS.append(self.path)
  if path=='/api/admin/plans':return self.reply({'data':{'plans':[PLAN]}})
  if path=='/api/admin/plans/'+PLAN['id']:return self.reply(PLAN)
  if path=='/api/admin/developer-portal/api-keys':return self.reply({'success':True,'data':{'api_keys':API_KEYS,'total':len(API_KEYS)}})
  if path=='/api/admin/developer-portal/stats':return self.reply({'success':True,'data':{'total_api_keys':len(API_KEYS),'active_api_keys':sum(k['status']=='active' for k in API_KEYS),'revoked_api_keys':sum(k['status']=='revoked' for k in API_KEYS),'expired_api_keys':0,'total_modules':0,'active_modules':0,'total_requests_today':0,'total_requests_this_month':0,'top_modules_by_usage':[]}})
  if path=='/api/admin/settings':return self.reply({'success':True,'data':SETTINGS})
  if path=='/api/admin/chat/stats':return self.reply({'success':True,'data':{'total_open':1,'total_in_progress':0,'total_resolved':0,'total_unassigned':1}})
  if path=='/api/admin/chat/topics':return self.reply({'success':True,'data':[{'id':PLAN['id'],'name':'support','label':'Support','is_active':True}]})
  if path=='/api/admin/chat/conversations':return self.reply({'success':True,'data':{'items':[CONVERSATION],'total':1,'page':1,'limit':20,'has_next':False}})
  if path=='/api/admin/chat/conversations/'+PLAN['id']:return self.reply({'success':True,'data':CONVERSATION})
  if path=='/api/admin/chat/conversations/'+PLAN['id']+'/messages':return self.reply({'success':True,'data':MESSAGES})
  if path.startswith('/api/admin/pay-orders'):
   order={'order_id':PLAN['id'],'plan_name':PLAN['name'],'wallet_address':'0x1111111111111111111111111111111111111111','amount':'5000000','token':'USDT','token_decimals':6,'status':'succeeded','payment_status':'succeeded','fulfillment_status':'pending','created_at':'2026-09-09T03:37:17Z','payment_available':True}
   return self.reply(order if path.endswith(PLAN['id']) else {'orders':[order],'next_offset':None})
  return self.reply({},404)
 def do_PUT(self):
  if not self.headers.get('Authorization','').startswith('Bearer '):return self.reply({},401)
  body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))));WRITES.append({'path':self.path,'body':body,'idempotency_key':self.headers.get('Idempotency-Key')})
  if self.path=='/api/admin/developer-portal/api-keys':
   key={'id':'33333333-3333-4333-8333-333333333333','key_prefix':'epsx_fixture','client_name':body['client_name'],'wallet_address':body['wallet_address'],'status':'active','total_requests':0,'ip_restrictions':[],'rate_limits':{'per_minute':10,'per_day':100},'allowed_modules':[],'permission_plans':[],'selected_permissions':[],'created_at':'2026-09-09T03:37:17Z','created_by':body['wallet_address'],'updated_at':'2026-09-09T03:37:17Z'};API_KEYS.append(key)
   return self.reply({'success':True,'data':{'api_key':key,'secret':'fixture-only-secret-never-a-production-key'}})
  if self.path.startswith('/api/admin/developer-portal/api-keys/'):
   key=API_KEYS[0]
   if self.path.endswith('/revoke'):key['status']='revoked'
   if self.path.endswith('/expiration'):key['expires_at']=body.get('expires_at')
   return self.reply({'success':True,'data':key})
  if self.path=='/api/admin/plans/'+PLAN['id']:
   PLAN.update({k:v for k,v in body.items() if v is not None});return self.reply(PLAN)
  if self.path.startswith('/api/admin/chat/conversations/'):
   if self.path.endswith('/messages'):MESSAGES.append({'id':'22222222-2222-4222-8222-222222222222','conversation_id':PLAN['id'],'sender_type':'agent','sender_address':CONVERSATION['wallet_address'],'content':body['content'],'is_read':False,'created_at':'2026-09-09T03:37:17Z'})
   elif self.path.endswith('/status'):CONVERSATION['status']=body['status']
   elif self.path.endswith('/assign'):CONVERSATION['assigned_agent']=body['agent_address']
   return self.reply({'success':True,'data':{}})
  if self.path.startswith('/api/admin/settings'):

   for item in body['settings']:SETTINGS[item['category']][item['key']]=item['value']
   return self.reply({'success':True,'data':body})
  return self.reply({},404)
 def do_POST(self):return self.do_PUT()
 def do_PATCH(self):return self.do_PUT()
def b64(b):return base64.urlsafe_b64encode(b).decode().rstrip('=')
def number(n):return b64(n.to_bytes((n.bit_length()+7)//8,'big'))
def main():
 global TOKEN,BASE,JWKS
 private=rsa.generate_private_key(public_exponent=65537,key_size=2048);public=private.public_key().public_numbers();JWKS={'keys':[{'kty':'RSA','use':'sig','alg':'RS256','kid':'admin-fixture','n':number(public.n),'e':number(public.e)}]}
 upstream=ThreadingHTTPServer(('127.0.0.1',0),Backend);threading.Thread(target=upstream.serve_forever,daemon=True).start()
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 BASE=f'http://127.0.0.1:{port}';backend=f'http://127.0.0.1:{upstream.server_port}';now=int(time.time())
 header=b64(json.dumps({'alg':'RS256','kid':'admin-fixture','typ':'JWT'}).encode());claims=b64(json.dumps({'iss':backend,'sub':'0x1111111111111111111111111111111111111111','aud':['epsx-admin'],'exp':now+1800,'iat':now-1,'jti':'isolated-admin-audit','scope':'openid permissions','wallet_address':'0x1111111111111111111111111111111111111111','auth_method':'web3_siwe','auth_time':now-1,'permissions':['admin:*:*']}).encode());signing=f'{header}.{claims}';TOKEN=signing+'.'+b64(private.sign(signing.encode(),padding.PKCS1v15(),hashes.SHA256()))
 public=OUT/f'fixture-public-{os.getpid()}'
 shutil.copytree(BUNDLE/'public',public,dirs_exist_ok=True)
 env=dict(os.environ,EPSX_ENV='development',ENV='development',IP='127.0.0.1',HOST='127.0.0.1',PORT=str(port),API_URL=backend,BACKEND_URL=backend,IDENTITY_SERVICE_URL=backend,CONTENT_SERVICE_URL=backend,OIDC_ISSUER=backend,OIDC_JWKS_URL=backend+'/.well-known/jwks.json',DIOXUS_PUBLIC_PATH=str(public))
 binary=OUT/f'fixture-server-{os.getpid()}'
 shutil.copy2(BUNDLE/'server',binary)
 with (OUT/'fixture-server.log').open('w') as log:
  process=subprocess.Popen([str(binary)],cwd=ROOT,env=env,stdout=log,stderr=subprocess.STDOUT)
  print(json.dumps({'login':backend+'/__fixture/login','app':BASE,'report':backend+'/__fixture/report'}),flush=True)
  try:print("Server exit:",process.wait(),flush=True)
  finally:
   process.terminate();upstream.shutdown();(OUT/'fixture-report.json').write_text(json.dumps({'reads':READS,'writes':WRITES},indent=2))
if __name__=='__main__':main()
