"""Isolated notification/payment mutation fixture, never sends real messages."""
import fullstack_browser as common
from urllib.parse import urlparse,parse_qs
import json
LINKS=[]
NOTIFICATIONS=[{'id':'notification-41','title':'Local fixture notification','subject':'Fixture subject','channel':'in_app','status':'sent','notification_type':'system','priority':'high','sent_at':'2026-09-09T03:37:17Z','created_at':'2026-09-09T03:37:17Z'}]
INTENT={'id':'pi_fixture','chain_id':'56','payer':'0x1111111111111111111111111111111111111111','payee':'0x2222222222222222222222222222222222222222','amount':'1000000','token_address':'0x3333333333333333333333333333333333333333','status':'pending','escrow_id':None,'tx_hash':None,'description':None,'expires_at':None,'created_at':'2026-09-09T03:37:17Z','updated_at':'2026-09-09T03:37:17Z'}
class Backend(common.Backend):
 def do_GET(self):
  path=urlparse(self.path).path;q=parse_qs(urlparse(self.path).query)
  if path.startswith('/api/v1/notification/admin/'):
   if not self.headers.get('Authorization','').startswith('Bearer '):return self.reply({},401)
   common.READS.append(self.path)
   if path.endswith('/metrics'):
    metrics={k:0 for k in ['queue_depth','queue_age_seconds','suppressed','retry_wait','terminal_failed','dead_lettered','provider_accepted','attempting','provider_events','delivery_attempts','replay_cursors','replay_cursor_age_seconds','active_streams','stream_connections_total','stream_reconnects_total','stream_replayed_events_total','stream_lag_seconds','stream_query_failures_total']};metrics['channel_outcomes']={'in_app':1};return self.reply(metrics)
   if path.endswith('/list'):
    rows=[r for r in NOTIFICATIONS if not q.get('status') or r['status']==q['status'][0]];offset=int(q.get('offset',['0'])[0]);return self.reply({'items':rows[offset:offset+20],'total':len(rows),'limit':20,'offset':offset})
  if path=='/api/v1/admin/pay/links':
   common.READS.append(self.path);return self.reply({'items':LINKS,'total':len(LINKS),'limit':100,'offset':0,'correlation_id':'fixture-payment-links'})
  if path=='/api/admin/plans/user-access/list':
   common.READS.append(self.path);return self.reply({'success':True,'data':{'users':[{'wallet_address':INTENT['payer'],'current_plan_id':common.PLAN['id'],'plan_name':'Fixture access','plan_expires_at':'2026-12-31T00:00:00Z','days_remaining':10,'status':'active'}],'pagination':{'page':int(q.get('page',['1'])[0]),'limit':int(q.get('limit',['20'])[0]),'total':1,'total_pages':1}}})
  if path=='/api/v1/admin/pay/intents':
   common.READS.append(self.path);limit=int(q.get('limit',['20'])[0]);offset=int(q.get('offset',['0'])[0]);rows=[INTENT] if not q.get('status') or INTENT['status']==q['status'][0] else [];return self.reply({'items':rows[offset:offset+limit],'total':len(rows),'limit':limit,'offset':offset})
  return super().do_GET()
 def do_POST(self):
  if self.path=='/api/v1/admin/pay/links':
   body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))));common.WRITES.append({'path':self.path,'body':body,'idempotency_key':self.headers.get('Idempotency-Key')});LINKS.append({'id':common.PLAN['id'],'slug':'fixture-link','intent_id':body['intent_id'],'max_uses':body.get('max_uses') or 0,'current_uses':0,'expires_at':None,'created_at':'2026-09-09T03:37:17Z','status':'active','version':1});return self.reply({})
  if self.path=='/api/v1/admin/pay/links/'+common.PLAN['id']+'/disable':
   body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))));common.WRITES.append({'path':self.path,'body':body,'idempotency_key':self.headers.get('Idempotency-Key')});LINKS[0]['status']='disabled';LINKS[0]['version']=2;return self.reply({})
  if self.path=='/api/v1/admin/pay/intents/pi_fixture/cancel':
   body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))));common.WRITES.append({'path':self.path,'body':body,'idempotency_key':self.headers.get('Idempotency-Key')});INTENT['status']='cancelled';return self.reply({})
  if self.path=='/api/v1/notification/send':
   body=json.loads(self.rfile.read(int(self.headers.get('Content-Length','0'))));common.WRITES.append({'path':self.path,'body':body,'idempotency_key':self.headers.get('Idempotency-Key')});return self.reply({'id':'idem_fixture_notification','status':'sent','delivered':True,'request_id':'fixture-notification'})
  if self.path.startswith('/api/v1/notification/admin/') and self.path.endswith('/read'):
   common.WRITES.append({'path':self.path,'body':{},'idempotency_key':None});return self.reply({})
  return super().do_POST()
 def do_DELETE(self):
  if self.path.startswith('/api/v1/notification/admin/'):
   common.WRITES.append({'path':self.path,'body':{},'idempotency_key':None});NOTIFICATIONS.clear();return self.reply({})
  return self.reply({},404)
common.Backend=Backend
if __name__=='__main__':common.main()
