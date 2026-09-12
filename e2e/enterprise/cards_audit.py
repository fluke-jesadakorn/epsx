"""Local-only Explore state/SSR audit. Run with the documented fixture preview.

Faults are always restored. Browser interactions and screenshots are separate.
"""
from pathlib import Path
from urllib.request import Request, urlopen
from urllib.error import HTTPError
import json, re
from rankings_audit import ProductText

OUT = Path(__file__).resolve().parents[2] / 'target/compact-enterprise-cards/after'
TOKEN = {'x-epsx-e2e-token':'epsx-enterprise-local-fixture', 'Content-Type':'application/json'}

def put(port, path, body):
    return json.load(urlopen(Request(f'http://127.0.0.1:{port}{path}',data=json.dumps(body).encode(),headers=TOKEN,method='PUT'),timeout=10))

def read(port, path):
    try: response=urlopen(f'http://127.0.0.1:{port}{path}',timeout=15)
    except HTTPError as error: response=error
    return response.status, response.read().decode()

def main():
    OUT.mkdir(parents=True,exist_ok=True); results=[]
    cases=[('healthy',[],3300),('analytics-empty',[],3300),('analytics-malformed',[],3300),('analytics-unavailable',[],3300),('analytics-limited',[],3300),('healthy',['filters-unavailable'],3300),('healthy',['rankings-restricted'],3300),('healthy',['rankings-restricted'],3301)]
    try:
        for mode,faults,port in cases:
            put(48081,'/__e2e/mode',{'mode':mode});put(48082,'/__qa/state',{'faults':faults})
            status,html=read(port,'/analytics');text=ProductText();text.feed(html);copy=' '.join(text.text)
            state=re.search('data-analytics-state="([^"]+)"',html).group(1)
            count=html.count('class="fe-ranking-card"');details_links=html.count('data-tradingview-details="true"')
            expected_cards=mode in ('healthy','analytics-limited') and 'rankings-restricted' not in faults
            assert bool(count)==bool(expected_cards),(mode,faults,count)
            assert count==details_links
            assert 'data-fe-company-dialog="true"' not in html
            assert 'fe-market-table' not in html and 'fe-mobile-market' not in html
            assert not re.search(r'\bEPS\b|\bGrowth\b|\bScore\b|\bPrice\b',copy,re.I)
            results.append({'mode':mode,'faults':faults,'port':port,'http':status,'state':state,'cards':count,'tradingview_links':details_links,'text':copy})
        for port in (3302,3303):
            status,html=read(port,'/')
            assert status==200 and '/public/enterprise.css' not in html and 'fe-company-dialog' not in html
            results.append({'smoke':port,'http':status,'frontendStylesAbsent':True})
        (OUT/'states-ssr.json').write_text(json.dumps(results,indent=2))
        print(json.dumps({'cases':len(cases),'smoke':2,'passed':True}))
    finally:
        put(48081,'/__e2e/mode',{'mode':'healthy'});put(48082,'/__qa/state',{'faults':[]})

if __name__=='__main__': main()
