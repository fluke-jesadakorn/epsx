"""Read-only link crawl of hydrated dev/fixture apps, including intermediate frames.
Usage: python3 .../dioxus_navigation_audit.py frontend http://127.0.0.1:3000
Uses an isolated browser session. Never signs a wallet or submits mutations.
"""
import json, os, re, sys, subprocess, time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from urllib.parse import urlparse, parse_qs
from urllib.request import urlopen, Request
from urllib.error import HTTPError
ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ.get('EPSX_NAV_OUT',str(ROOT / 'target/navigation-audit')))
APP = sys.argv[1]
BASE = sys.argv[2].rstrip('/')
SESSION = 'epsx-navigation-' + APP
SHELL = {'frontend': '.epsx-frontend', 'admin': '.admin-app-shell', 'pay': '.epsx-merchant'}[APP]

def browser(*args):
    p = subprocess.run(['agent-browser', '--session', SESSION, *args], capture_output=True, text=True, timeout=45)
    if p.returncode: raise RuntimeError(p.stdout + p.stderr)
    return p.stdout.strip()
def evaluate(script): return json.loads(browser('eval', script))
def settled():
    browser('wait', '--fn', 'window.__epsxNavigationLifecycle === true && !document.querySelector("[data-page-skeleton]") && !!document.querySelector(' + json.dumps(SHELL) + ')')
    # Ensure effects and the frame observer have run, without a fixed network sleep.
    evaluate('(async()=>{await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));return true})()')
def links():
    return evaluate('Array.from(document.querySelectorAll("a[href]")).map(a=>a.getAttribute("href")).filter(h=>h.startsWith("/")&&!h.startsWith("//")&&!/^\\/(api|_server|public|runtime|assets)\\//.test(h))')
def direct(path):
    try:
        request = Request(BASE + path)
        if os.environ.get('EPSX_NAV_TOKEN'): request.add_header('Cookie', f'epsx.{APP}.access_token=' + os.environ['EPSX_NAV_TOKEN'])
        with urlopen(request, timeout=30) as response: status, body = response.status, response.read().decode()
    except HTTPError as error: status, body = error.code, error.read().decode()
    assert '<html' in body and 'id="main"' in body, (path, status, 'SSR document missing')
    assert 'data-page-skeleton="true"' not in body, (path, 'SSR returned unresolved skeleton')
    return status

def route_samples():
    if APP == 'pay':
        return ['/','/dashboard','/payments','/payments/pi_fixture','/packages','/packages/pkg_fixture','/packages/pkg_fixture/edit','/payment-links','/webhooks','/settings','/escrow','/m/m_fixture','/checkout/cs_fixture','/intent/pi_fixture','/r/plink_fixture','/r/ref','/docs','/docs/merchant']
    source=(ROOT/'shared/rust/dioxus_ui/src/routes.rs').read_text()
    enum='FrontendRoute' if APP=='frontend' else 'AdminRoute'
    source=source.split('pub enum '+enum+' {',1)[1].split('\n#[component]',1)[0]
    paths=[]
    for route in re.findall(r'#\[route\("([^"\n]+)"\)\]',source):
        path=route.split('?')[0]
        for name,value in [('..route','__navigation_missing__'),('address','0x'+'1'*40),('order_id','11111111-1111-4111-8111-111111111111'),('ptype','plan'),('pid','11111111-1111-4111-8111-111111111111'),('slug','fixture-story'),('id','11111111-1111-4111-8111-111111111111')]:
            path=path.replace(':'+name,value)
        paths.append(path)
    return list(dict.fromkeys(paths))

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    records, seen, edges = [], set(), []
    try:
        browser('cookies','clear')
        if os.environ.get('EPSX_NAV_TOKEN'): browser('cookies','set',f'epsx.{APP}.access_token',os.environ['EPSX_NAV_TOKEN'],'--url',BASE,'--httpOnly')
        browser('errors','--clear')
        if os.environ.get('EPSX_NAV_DEEP_ONLY'):
            deep = []
            for path in route_samples():
                browser('open', BASE + path)
                settled()
                evaluate('window.__deepDoc=document;window.__deepShell=document.querySelector(' + json.dumps(SHELL) + ');true')
                landed = evaluate('location.pathname+location.search')
                evaluate('(document.querySelector(".site-nav-brand") || Array.from(document.querySelectorAll("a[href]")).find(a=>a.getAttribute("href")==="/")).click();true')
                settled()
                assert evaluate('document===window.__deepDoc && window.__deepShell.isConnected'), path
                deep.append({'path':path,'hydrated_at':landed,'brand_link_kept_document_and_shell':True})
            errors=browser('errors')
            assert not errors.strip(), errors
            (OUT/(APP+'-deep-hydration.json')).write_text(json.dumps(deep,indent=2)+'\n')
            print(json.dumps({'app':APP,'direct_hydration_routes':len(deep),'page_errors':errors}))
            return
        start = '/wallet-management/wallets?page=2&limit=25&status=active' if APP == 'admin' and os.environ.get('EPSX_NAV_TOKEN') else '/'
        browser('open', BASE + start)
        settled()
        evaluate('window.__navDoc=document;window.__navShell=document.querySelector(' + json.dumps(SHELL) + ');window.__navTheme=window.__navShell.classList.contains("dark");window.__navThemeFlashes=0;window.__navBlankFrames=0;window.__navFrames=0;window.__navSkeletonFrames=0;window.__navObserve=true;(function tick(){if(!window.__navObserve)return;window.__navFrames++;if(window.__navShell.classList.contains("dark")!==window.__navTheme)window.__navThemeFlashes++;if(document.querySelector("[data-page-skeleton]"))window.__navSkeletonFrames++;if(!window.__navShell.isConnected||!window.__navShell.getBoundingClientRect().height)window.__navBlankFrames++;requestAnimationFrame(tick)})();true')
        evaluate('window.__navFetch=window.fetch;window.fetch=async function(...args){const url=String(args[0]?.url||args[0]);if(url.includes("/_server/"))await new Promise(r=>setTimeout(r,250));return window.__navFetch.apply(this,args)};true')
        evaluate('window.__navChanges=0;for(const name of ["pushState","replaceState"]){const original=history[name].bind(history);history[name]=function(...args){const before=location.href;const result=original(...args);if(location.href!==before)window.__navChanges++;return result}};true')
        form_checks = []
        if APP == 'admin' and os.environ.get('EPSX_NAV_TOKEN'):
            assert evaluate('document.querySelector("select[name=status]").value') == 'active'
            browser('fill','input[name=search]','0x1111')
            browser('find','role','button','click','--name','Search')
            browser('wait','--fn','location.search.includes("search=0x1111") && location.search.includes("page=1")')
            settled()
            assert evaluate('document.querySelector("input[name=search]").value === "0x1111" && document.querySelector("select[name=status]").value === "active" && document.querySelector("input[name=limit]").value === "25"')
            browser('wait','--fn','Array.from(document.querySelectorAll("a")).some(a=>a.textContent==="Next" && a.search.includes("page=2"))')
            browser('find','role','link','click','--name','Next')
            browser('wait','--fn','location.search.includes("page=2") && location.search.includes("limit=25")')
            settled()
            browser('back')
            browser('wait','--fn','location.search.includes("page=1")')
            settled()
            assert evaluate('document===window.__navDoc && document.querySelector("input[name=search]").value === "0x1111" && document.querySelector("select[name=status]").value === "active"')
            form_checks.append('Wallet search/status/limit survive submit, pagination and Back without replacing document')
            evaluate('Array.from(document.querySelectorAll("a")).find(a=>a.textContent==="Next").click();history.back();true')
            browser('wait','--fn','location.search.includes("page=1")')
            settled()
            evaluate('(async()=>{await new Promise(r=>setTimeout(r,600));return true})()')
            assert evaluate('document===window.__navDoc && Array.from(document.querySelectorAll("a")).some(a=>a.textContent==="Next" && a.search.includes("page=2"))'), 'Late page response replaced current pagination'
            form_checks.append('Rapid Next/Back keeps page 1 after the delayed page 2 request completes')
        start = evaluate('location.pathname+location.search+location.hash')
        queue = [(start, start)]
        while queue and len(seen) < 100:
            source, target = queue.pop(0)
            if target in seen: continue
            current = evaluate('location.pathname+location.search+location.hash')
            if current != source:
                # Browser history returns to the saved source without a document load.
                # Traverse only actual links in the current rendered UI.
                available = links()
                if target not in available:
                    edges.append({'from': source, 'to': target, 'status': 'not-currently-reachable'})
                    continue
            if current != target:
                changes = evaluate('window.__navChanges')
                clicked = evaluate('(()=>{const a=Array.from(document.querySelectorAll("a[href]")).find(a=>a.getAttribute("href")===' + json.dumps(target) + ');if(!a)return false;a.click();return true})()')
                if not clicked: continue
                browser('wait', '--fn', 'window.__navChanges > ' + str(changes))
                settled()
            assert evaluate('!Array.from(document.scripts).some(s=>s.src.includes("epsx_browser_runtime")||s.src.includes("epsx-browser-runtime"))'), 'legacy runtime loaded'
            assert evaluate('document===window.__navDoc && window.__navShell.isConnected'), target
            assert evaluate('window.__navBlankFrames===0'), (target, 'blank frame')
            landed = evaluate('location.pathname+location.search+location.hash')
            aliases = {'/manual': '/analytics', '/pricing': '/plans', '/notifications': '/notifications/manage'}
            auth_target = parse_qs(urlparse(target).query).get('return_url', ['/dashboard' if APP == 'admin' else '/'])[0] if urlparse(target).path in ['/auth','/admin/auth'] and os.environ.get('EPSX_NAV_TOKEN') else None
            guest_guard = APP == 'admin' and not os.environ.get('EPSX_NAV_TOKEN') and urlparse(landed).path == '/auth'
            assert guest_guard or landed == auth_target or landed == target or landed.split('?')[0] == aliases.get(target.split('?')[0]) or (target.startswith('/portfolio/') and landed == '/portfolio'), (target, landed)
            seen.add(target)
            records.append({'path': target, 'landed': landed, 'ssr_status': direct(target.split('#')[0]), 'same_document': True, 'persistent_shell': True})
            available = list(dict.fromkeys(links()))
            # Depth-first follows real edges; queue includes sibling/global links too.
            queue = [(landed, href) for href in available if href not in seen] + queue
        assert len(records) >= (3 if APP == "admin" and not os.environ.get("EPSX_NAV_TOKEN") else 5), ('insufficient route coverage', records)
        if len(records) > 1:
            browser('back')
            settled()
            assert evaluate('document===window.__navDoc && window.__navShell.isConnected'), 'Back replaced document/shell'
            browser('forward')
            settled()
            assert evaluate('document===window.__navDoc && window.__navShell.isConnected'), 'Forward replaced document/shell'
        modified = evaluate('(()=>{const a=Array.from(document.querySelectorAll("a[href]")).find(a=>a.getAttribute("href").startsWith("/"));return [{ctrlKey:true},{metaKey:true},{shiftKey:true},{button:1}].map(extra=>{let intercepted=null;document.addEventListener("click",e=>{intercepted=e.defaultPrevented;e.preventDefault()},{once:true});a.dispatchEvent(new MouseEvent("click",{bubbles:true,cancelable:true,button:0,...extra}));return intercepted})})()')
        assert modified == [False,False,False,False], ('Modified clicks intercepted',modified)
        browser('set', 'viewport', '390', '844')
        if evaluate('!!document.querySelector(".site-nav-mobile > summary")'):
            browser('click', '.site-nav-mobile > summary')
            mobile_target = evaluate('Array.from(document.querySelectorAll(".site-nav-mobile a[href]")).map(a=>a.getAttribute("href")).find(h=>h.startsWith("/")&&h!==location.pathname+location.search)||null')
            if mobile_target:
                browser('click', '.site-nav-mobile a[href=' + json.dumps(mobile_target) + ']')
                settled()
                assert evaluate('document===window.__navDoc && window.__navShell.isConnected'), 'Mobile menu replaced document/shell'
                assert evaluate('!document.querySelector(".site-nav-mobile").open'), 'Mobile menu stayed open'
        assert evaluate('document.documentElement.scrollWidth<=innerWidth'), 'mobile overflow'
        browser('screenshot', str(OUT / (APP + '-mobile.png')))
        browser('set', 'viewport', '1440', '1000')
        browser('screenshot', str(OUT / (APP + '-desktop.png')))
        with ThreadPoolExecutor(max_workers=3) as pool:
            ssr = list(pool.map(lambda path: {'path': path, 'status': direct(path)}, route_samples()))
        (OUT / (APP + '-ssr.json')).write_text(json.dumps(ssr,indent=2)+'\n')
        result = {'app': APP, 'form_navigation_checks': form_checks, 'modified_clicks_keep_browser_default': modified == [False,False,False,False], 'routes': records, 'unvisited_edges': edges,
                  'page_errors': browser('errors'), 'theme_flash_frames': evaluate('window.__navThemeFlashes'), 'frames_observed': evaluate('window.__navFrames'), 'skeleton_frames': evaluate('window.__navSkeletonFrames'), 'blank_frames': evaluate('window.__navBlankFrames')}
        assert result['theme_flash_frames'] == 0, 'Theme changed during navigation'
        assert not result['page_errors'].strip(), result['page_errors']
        (OUT / (APP + '-browser.json')).write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps({'app': APP, 'routes': len(records), 'blank_frames': result['blank_frames']}))
    finally:
        browser('close')
if __name__ == '__main__': main()
