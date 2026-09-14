"""Explicit, loopback-only v2 review fixtures matching current read contracts."""
from datetime import date, timedelta
from urllib.parse import parse_qs

OWNER = '0xea6400000000000000000000000000000000e3df'
STAMP = '2026-09-08T00:00:00Z'
PLAN_ID = '00000000-0000-0000-0000-000000000001'


def plan(index, name, price, features):
    return dict(id=f'00000000-0000-0000-0000-{index:012}', name=name,
        plan_type='subscription', current_price=str(price), effective_price=float(price),
        promotion_active=False, promotion_status='inactive', promotion_discount=0.0,
        promotion_ends_at=None, currency='USD', billing_cycle='monthly', features=features,
        permissions=['epsx:analytics:view'], is_active=True, tier_level=index,
        plan_group='standard', ranking_offset=0, rankings_limit=100,
        checkout_price=str(price), settlement_currency='USDC', duration_days=30)


PLANS = [
    plan(1,'Explore',12,['Reported company figures','Country and sector filters','Personal watchlist']),
    plan(2,'Research',29,['Reported company figures','Quarterly performance details','Watchlist groups','Developer API access']),
    plan(3,'Professional',59,['Reported company figures','Quarterly performance details','Watchlist groups','Developer API access','Account support']),
]


def read_fixture(path, query):
    if path == '/api/public/plans':
        return dict(success=True, data=PLANS)
    if path.startswith('/api/public/plans/'):
        return dict(success=True, data=next((p for p in PLANS if p['id']==path.rsplit('/',1)[-1]),None))
    if path == '/api/users/profile':
        return dict(success=True, data=dict(wallet_address=OWNER, permissions=['epsx:analytics:view'],
            auth_method='siwe', created_at='2026-07-01T00:00:00Z', last_login=STAMP))
    if path == '/api/users/access-overview':
        return dict(success=True, data=dict(current_tier='Research', direct_permissions=[], groups=[
            dict(id=PLAN_ID,name='Research',description='Local review fixture',expires_at='2026-10-08T00:00:00Z',
                 permissions=['epsx:analytics:view'],source_type='subscription',assigned_at=STAMP,
                 assigned_by=None,days_remaining=30,can_renew=True,renewal_price=29,billing_cycle='monthly',tier_level=2)]))
    if path == '/api/payments/credits/balance':
        return dict(wallet_address=OWNER,balance='120.00',pending_balance='0.00',
            available_balance='120.00',lifetime_earned='150.00',lifetime_spent='30.00',last_transaction_at=STAMP)
    if path == '/api/payments/credits/history':
        return dict(success=True,count=1,data=[dict(id=PLAN_ID,wallet_address=OWNER,amount=120,
            balance_after=120,tx_type='grant',reference_id=None,reference_type='review_fixture',
            reason='Local review credit',granted_by=None,expires_at=None,created_at=STAMP)])
    if path == '/api/payments/history':
        return dict(success=True,data=dict(payments=[],pagination=dict(page=1,per_page=10,total=0,total_pages=0)))
    if path == '/api/developer-portal/overview':
        days=int(parse_qs(query).get('days',['30'])[0])
        daily=[dict(date=(date(2026,9,8)-timedelta(days=days-i-1)).isoformat(),
                    total_requests=20+i%7,error_requests=0) for i in range(days)]
        total=sum(d['total_requests'] for d in daily)
        return dict(success=True,data=dict(
            entitlement=dict(plans=[dict(id=PLAN_ID,name='Research',slug='research',expires_at=None)],
                assignable_scopes=['epsx:analytics:view'],
                rate_limits=dict(per_minute=60,per_hour=1000,per_day=10000,burst=10),
                can_read=True,can_write=True,has_active_api_entitlement=True),
            api_keys=[dict(id=PLAN_ID,key_prefix='epsx_fixture…',name='Research workspace',
                description='Local review key — no secret or credential exists',status='active',
                scopes=['epsx:analytics:view'],total_requests=total,expires_at=None,last_used_at=STAMP,created_at=STAMP)],
            total_api_keys=1,usage=dict(days=days,total_requests=total,successful_requests=total,
                error_requests=0,success_rate=100.0,error_rate=0.0,average_response_time_ms=84.0,
                daily=daily,top_endpoints=[dict(endpoint='/api/analytics/rankings',method='GET',
                    request_count=total,error_count=0,average_response_time_ms=84.0)])))

    if path == '/api/v1/notification/preferences':
        return dict(channels=dict(email=True,in_app=True,push=False),quiet_hours=None,timezone='UTC',updated_at=STAMP)
    if path == '/api/v1/notification/unread-count':
        items=read_fixture('/api/v1/notification/list',query)['items']
        return dict(count=sum(item['read_at'] is None for item in items))
    if path == '/api-docs/openapi.json':
        return dict(openapi='3.1.0',info=dict(title='EPSX local review API',version='1.0'),paths={'/api/analytics/rankings':{'get':{
            'operationId':'getRankings','summary':'Read reported company data',
            'x-epsx-required-scopes':['epsx:analytics:view'],'x-epsx-api-key-callable':True,
            'x-epsx-mutation':False,'x-epsx-idempotent':False}}})
    if path == '/api/v1/notification/list':
        items=[
            dict(id='550e8400-e29b-41d4-a716-446655440001',subject='Welcome to your workspace',
                 title='Welcome to your workspace',body='Explore company data and save your first watchlist.',
                 created_at=STAMP,read_at=None,notification_type='system',priority='normal',action_url=None),
            dict(id='550e8400-e29b-41d4-a716-446655440002',subject='Your account is ready',
                 title='Your account is ready',body='You can review your profile and access from Account.',
                 created_at='2026-09-07T10:00:00Z',read_at=STAMP,notification_type='system',priority='normal',action_url=None)
        ]
        for item in items:
            item.update(user_id=OWNER,recipient=OWNER,channel='in_app',status='sent',
                template_id=None,data=None,error=None,sent_at=item['created_at'],clicked_at=None,expires_at=None)
        params=parse_qs(query)
        status=params.get('status',[None])[0]
        if status == 'unread': items=[item for item in items if item['read_at'] is None]
        elif status == 'read': items=[item for item in items if item['read_at'] is not None]
        elif status and status != 'all': items=[item for item in items if item['status']==status]
        for key,field in [('type','notification_type'),('priority','priority')]:
            selected=params.get(key,[None])[0]
            if selected: items=[item for item in items if item[field]==selected]
        total=len(items); offset=int(params.get('offset',['0'])[0]);limit=int(params.get('limit',['50'])[0])
        return dict(items=items[offset:offset+limit],total=total)
    if path == '/api/public/news' or path.startswith('/api/public/news/'):
        articles=[
            dict(id='550e8400-e29b-41d4-a716-446655440100',slug='deterministic-market-brief',
                 title='A clearer way to explore company data',summary='A local preview of the EPSX reading experience.',
                 content='## Reading the figures\n\nThis is a local interface review article. EPS figures describe reported earnings per share.',
                 author='EPSX',status='published',published_at=STAMP,tags=['product'],featured=False),
            dict(id='550e8400-e29b-41d4-a716-446655440101',slug='understanding-reporting-periods',
                 title='Understanding reporting periods',summary='Keep reported quarters and estimates in perspective.',
                 content='Local review article for the news layout.',author='EPSX',
                 status='published',published_at='2026-09-07T10:00:00Z',tags=['updates'],featured=False)
        ]
        return dict(success=True,data=dict(articles=articles,total=2,page=1,limit=100)) if path=='/api/public/news' else dict(success=True,data=articles[0])
    if path == '/api/chat/inbox' or path.startswith('/api/chat/conversations/'):
        topic=dict(id=PLAN_ID,name='general',label='General support',description='Questions about EPSX',
                   icon='message-circle',sort_order=0,is_active=True,created_at=STAMP)
        conversation=dict(id='550e8400-e29b-41d4-a716-446655440000',topic_id=PLAN_ID,wallet_address=OWNER,
            subject='Getting started with company data',status='open',assigned_agent=None,last_message_at=STAMP,
            unread_user=1,unread_agent=0,metadata={},created_at=STAMP,updated_at=STAMP)
        if path=='/api/chat/inbox':
            return dict(success=True,data=dict(topics=[topic],conversations=[conversation]))
        return dict(success=True,data=dict(conversation=conversation,messages=[
            dict(id='550e8400-e29b-41d4-a716-446655440003',conversation_id=conversation['id'],
                 sender_type='user',sender_address=OWNER,content='Where can I find quarterly EPS?',
                 is_read=True,metadata={},created_at='2026-09-07T10:00:00Z'),
            dict(id='550e8400-e29b-41d4-a716-446655440004',conversation_id=conversation['id'],
                 sender_type='agent',sender_address=None,content='Open a company in Explore to review its reporting periods.',
                 is_read=False,metadata={},created_at=STAMP)]))
    return None
