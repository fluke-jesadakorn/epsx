"""Read-only native SSR route/copy audit for the Company Rankings preview.

Start the loopback fixture and preview as documented in README.md first.
Browser interactions and responsive screenshots are reviewed separately.
"""
from pathlib import Path
from urllib.request import urlopen
from urllib.error import HTTPError
from html.parser import HTMLParser
import json
import re

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / 'target/company-rankings-next-action/after'
ROUTES = ['/', '/analytics', '/portfolio', '/dashboard', '/account', '/profile',
    '/permissions', '/account/credits', '/payment', '/plans', '/notifications',
    '/chat', '/chat/history', '/developer', '/developer/docs', '/developer/usage',
    '/news', '/manual', '/about', '/contact', '/terms', '/privacy', '/offline',
    '/access-denied', '/missing-page', '/auth?return_url=%2Fanalytics',
    '/pricing?ref=rankings-qa', '/portfolio/0xea6400000000000000000000000000000000e3df',
    '/chat/550e8400-e29b-41d4-a716-446655440000', '/news/deterministic-market-brief',
    '/payment/plan/00000000-0000-0000-0000-000000000001', '/index',
    '/profile?tab=account', '/profile?tab=email', '/profile?tab=data', '/chat?new=1',
    '/notifications?status=unread', '/notifications?status=read', '/developer/usage?days=7']
PUBLIC = {'/','/index','/analytics','/plans','/about','/manual','/contact','/terms','/privacy','/offline','/auth','/pricing'}

class ProductText(HTMLParser):
    def __init__(self):
        super().__init__(); self.depth=0; self.text=[]
    def handle_starttag(self, tag, attrs):
        if tag in ('script','style'): self.depth+=1
        if not self.depth:
            values=dict(attrs)
            self.text.extend(v for k,v in attrs if k in ('title','aria-label','alt') and v)
            if tag=='meta' and values.get('name') in ('description','keywords'):
                self.text.append(values.get('content',''))
    def handle_endtag(self,tag):
        if tag in ('script','style'): self.depth=max(0,self.depth-1)
    def handle_data(self,value):
        if not self.depth: self.text.append(value)

def main():
    OUT.mkdir(parents=True,exist_ok=True)
    records=[]
    for route in ROUTES:
        base='http://127.0.0.1:'+('3300' if route.split('?')[0] in PUBLIC else '3301')
        try: response=urlopen(base+route,timeout=15)
        except HTTPError as error: response=error
        html=response.read().decode(); parser=ProductText();parser.feed(html)
        text=' '.join(parser.text)
        # Exact backend-owned fixture content, retained verbatim by the product.
        owned_phrases = ['Personal watchlist', 'Watchlist groups',
            'Explore company data and save your first watchlist.',
            'Where can I find quarterly EPS?']
        retained = [phrase for phrase in owned_phrases if phrase in text]
        product_text = text
        for phrase in retained: product_text = product_text.replace(phrase, '')
        terms=re.findall(r'\bEPS\b|\bEPS growth\b|\bearnings per share\b|\bwatchlist\b',product_text,re.I)
        records.append({'route':route,'status':response.status,'url':response.url,
            'frontend_styles': '/public/enterprise.css?v=' in html,
            'mains':len(re.findall(r'<main[ >]',html)),
            'legacy_terms':terms,'backend_owned_text_retained':retained,
            'content_owner_exception':route.startswith(('/terms','/privacy','/news/','/developer/docs'))})
    (OUT/'routes-copy.json').write_text(json.dumps(records,indent=2))
    failures=[r for r in records if r['status'] != (404 if r['route']=='/missing-page' else 200) or not r['frontend_styles'] or r['mains']!=1 or (r['legacy_terms'] and not r['content_owner_exception'])]
    print(json.dumps({'routes':len(records),'failures':failures},indent=2))
    if failures: raise SystemExit(1)

if __name__=='__main__': main()
