"""Exercise the existing Rust analytics fixture modes through native SSR."""
from pathlib import Path
from urllib.request import Request, urlopen
import json
from capture import browser, ROOT

OUT = ROOT / 'target/enterprise-redesign/states'


def mode(value):
    request = Request('http://127.0.0.1:48081/__e2e/mode',
        data=json.dumps({'mode':value}).encode(), method='PUT',
        headers={'content-type':'application/json','x-epsx-e2e-token':'epsx-enterprise-local-fixture'})
    with urlopen(request,timeout=10) as response:
        assert response.status == 200


OUT.mkdir(parents=True,exist_ok=True)
records=[]
try:
    browser('--args','--disable-gpu','open','http://127.0.0.1:3300/analytics')
    browser('set','viewport','1440','900')
    browser('set','media','light','reduced-motion')
    for fixture,state in [('analytics-empty','empty'),('analytics-malformed','malformed'),
                          ('analytics-unavailable','unavailable'),('analytics-limited','ready')]:
        mode(fixture)
        browser('open','http://127.0.0.1:3300/analytics')
        actual=browser('eval', '''(() => ({
            state:document.querySelector('[data-analytics-state]')?.getAttribute('data-analytics-state'),
            symbols:Array.from(document.querySelectorAll('.fe-market-table [data-symbol]')).map(e=>e.getAttribute('data-symbol')),
            message:document.querySelector('main')?.innerText
        }))()''', data=True)['result']
        assert actual['state'] == state, actual
        if fixture=='analytics-limited':
            assert 'NVDA' not in actual['symbols'] and 'Rank 1 locked' in actual['message'], actual
        else:
            assert not actual['symbols'], actual
        browser('screenshot',str(OUT/f'{fixture}.png'))
        records.append({'fixture':fixture,'state':state,'symbols':actual['symbols'],'ok':True})
        (OUT/'results.json').write_text(json.dumps(records,indent=2))
finally:
    mode('healthy')
print(json.dumps(records,indent=2))
