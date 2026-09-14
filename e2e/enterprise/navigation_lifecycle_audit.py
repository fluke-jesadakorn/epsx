"""Isolated browser checks for scroll/focus lifecycle; no application/backend."""
import json, subprocess, threading
from pathlib import Path
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
ROOT=Path(__file__).resolve().parents[2]
SCRIPT=(ROOT/'shared/rust/dioxus_ui/src/navigation_lifecycle.js').read_text()
class Page(BaseHTTPRequestHandler):
 def log_message(self,*_):pass
 def do_GET(self):
  body=('<!doctype html><html><body><main id="epsx-main-content" tabindex="-1" style="min-height:3000px"><h1>Page</h1></main><script>history.scrollRestoration="manual";'+SCRIPT+'</script></body></html>').encode()
  self.send_response(200);self.send_header('Content-Type','text/html');self.end_headers();self.wfile.write(body)
def browser(*args):
 p=subprocess.run(['agent-browser','--session','epsx-navigation-lifecycle',*args],capture_output=True,text=True,timeout=35)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
def ev(script):return json.loads(browser('eval',script))
def frames():ev('(async()=>{await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));return true})()')
def main():
 server=ThreadingHTTPServer(('127.0.0.1',0),Page);threading.Thread(target=server.serve_forever,daemon=True).start();checks=[]
 try:
  browser('open',f'http://127.0.0.1:{server.server_port}/first')
  ev('window.__lifecycleDocument=document;scrollTo(0,600);true');frames()
  ev('history.replaceState([0,600],"",location.href);history.pushState([0,600],"","/first?limit=25");scrollTo(0,0);true');frames()
  assert ev('scrollY')==600;checks.append('Query navigation retains offset after Dioxus scroll reset')
  ev('history.pushState([0,600],"","/second");document.querySelector("main").innerHTML="<section data-page-skeleton=true>Loading</section>";true');frames()
  assert ev('scrollY')==0
  ev('document.querySelector("main").innerHTML="<h1>Second page</h1>";true');frames()
  assert ev('document.activeElement.id')=='epsx-main-content';checks.append('New page scrolls to top and focuses content after skeleton')
  browser('back');frames();assert ev('scrollY')==600;checks.append('Back restores history coordinates')
  browser('forward');frames();assert ev('document===window.__lifecycleDocument');checks.append('Forward retains same document')
  ev('history.pushState([0,0],"","/third#details");document.querySelector("main").innerHTML="<section data-page-skeleton=true>Loading</section>";true');frames()
  ev('document.querySelector("main").innerHTML="<div style=height:1200px></div><h2 id=details>Details</h2>";true');frames()
  assert ev('scrollY')>=1200;checks.append('Fragment scroll waits for destination content')
  out=ROOT/'target/navigation-audit';out.mkdir(parents=True,exist_ok=True);(out/'lifecycle-browser.json').write_text(json.dumps(checks,indent=2)+'\n');print(json.dumps(checks))
 finally:browser('close');server.shutdown()
if __name__=='__main__':main()
