"""Run against the ignored Pay browser fixture on loopback :43127.
Requires the built real Dioxus WASM/server assets; never contacts production.
"""
import json,subprocess,time,pathlib
PREFIX=['agent-browser','--session','epsx-pay-fullstack']
pathlib.Path("target/pay-fullstack-evidence").mkdir(parents=True,exist_ok=True)
checks=[]
def command(*args):
 p=subprocess.run(PREFIX+list(args),text=True,capture_output=True,timeout=40)
 if p.returncode:raise RuntimeError(p.stdout+p.stderr)
 return p.stdout.strip()
def evaluate(js):return json.loads(command('eval',js))
def check(name,condition):
 if not condition:raise AssertionError(name)
 checks.append(name)
def find(role,name,action,*args):
 if role=='link' and name in {'Overview','Payments','Packages','Payment links','Webhooks','Settings'}:
  if evaluate('!!document.querySelector(".site-nav-desktop .site-nav-group") && !document.querySelector(".site-nav-desktop .site-nav-group").open'):
   command('click','.site-nav-desktop .site-nav-group:first-child > summary')
 return command('find','role',role,action,'--name',name,*args)
def field(name,value):command('find','label',name,'fill',value)
def ready(text):command('wait','--text',text)
command('set','viewport','1440','1000')
command('open','http://127.0.0.1:43127/dashboard')
command('wait','--fn','document.querySelector(".epsx-merchant") !== null')
time.sleep(1)
command('eval','sessionStorage.clear();window.__payDocument="preserved";window.__payErrors=[];window.addEventListener("error",e=>window.__payErrors.push(e.message)); true')
dark=evaluate('!!document.querySelector(".epsx-merchant.dark")')
find('button','Toggle theme','click')
command('wait','--fn',f'!!document.querySelector(".epsx-merchant.dark") === {str(not dark).lower()}')
check('hydrated theme event',True)
find('link','Settings','click');ready('Shop details')
check('internal navigation keeps document',evaluate('window.__payDocument === "preserved"'))
check('theme persists across navigation',evaluate(f'!!document.querySelector(".epsx-merchant.dark") === {str(not dark).lower()}'))
field('Shop name','Hydrated merchant');find('button','Save shop name','click');ready('Saved.')
check('typed mutation updates page without reload',evaluate('document.querySelector(".md-context-bar").textContent.includes("Hydrated merchant") && window.__payDocument === "preserved"'))
field('Key name','Browser fixture');find('button','Create API key','click');ready('fixture_only_secret');find('button','I saved it','click')
check('one-time secret dismissed',evaluate('!document.body.textContent.includes("fixture_only_secret")'))
find('link','Packages','click');ready('Your catalog')
field('Package name','Reactive package');field('USDT price (optional)','7.50');field('Service duration in days (optional)','14');field('Description and service terms','Browser fixture only');find('button','Save package','click');ready('Reactive package')
check('catalog form is reactive',evaluate('document.querySelector(".md-products").textContent.includes("Reactive package") && window.__payDocument === "preserved"'))
command('select','select[aria-label="Environment"]','live');command('wait','--url','**environment=live')
check('environment updates URL without reload',evaluate('window.__payDocument === "preserved"'))
command('back');command('wait','--url','**environment=test')
check('history restores environment',evaluate('document.querySelector("select[aria-label=Environment]").value === "test"'))
find('link','View storefront ↗','click');ready('Choose a package')
command('find','first','article.md-product button','click');command('wait','--url','**/checkout/cs_fixture*');ready('QR / Transfer')
command('wait','--fn','document.querySelector(".pc-qr img").naturalWidth > 0')
check('payment QR image loads',True)
check('checkout palette applies to Dioxus root',evaluate('getComputedStyle(document.querySelector(".epsx-checkout")).getPropertyValue("--pay-bg").trim().length > 0'))
check('checkout navigation stays hydrated',evaluate('window.__payDocument === "preserved"'))
# These controls only affect the loopback fixture and its simulated wallet.
from urllib.request import Request,urlopen
BASE='http://127.0.0.1:43127'
def scenario(name):
 with urlopen(Request(BASE+'/fixture/scenario/'+name,method='POST')) as r:return json.load(r)
def sent():
 with urlopen(BASE+'/fixture/wallet') as r:return json.load(r)['sent']
def checkout(name):
 command('open',BASE+'/checkout/cs_'+name+'#token='+'b'*64)
 ready('Pay 5 USDT')
def stage(name):command('wait','--fn',f'document.querySelector("[data-checkout-stage={name}]") !== null')
scenario('wallet');checkout('progress')
before=sent();find('button','Pay 5 USDT','click');ready('Confirm in your wallet')
check('wallet request uses a distinct card',evaluate('!document.body.textContent.includes("awaiting_payment")'))
ready('Confirming payment');check('wallet submits exactly once',sent()==before+1)
check('transaction has one copy/explorer row',evaluate('document.querySelectorAll(".pc-transaction").length===1 && document.querySelector(".pc-transaction a").href.startsWith("https://bscscan.com/tx/")'))
command('reload');ready('Confirming payment')
check('refresh restores pending without a payment button',evaluate('!Array.from(document.querySelectorAll("button")).some(b=>b.textContent.includes("Pay 5 USDT"))') and sent()==before+1)
scenario('outage');time.sleep(5);check('outage preserves submitted payment',sent()==before+1)
scenario('paid');ready('Activating your plan');time.sleep(5)
check('no redirect before actual grant',evaluate('location.pathname==="/checkout/cs_progress"'))
scenario('granted');command('wait','--url','**/account/payments/00000000-0000-0000-0000-000000000000');ready('Purchase details')
check('ready grant redirects to verified purchase',True)
command('open',BASE+'/checkout/cs_progress#token='+'b'*64);ready('Your plan is ready');time.sleep(5)
check('redirect happens once per flow',evaluate('location.pathname==="/checkout/cs_progress"'))
command('open',BASE+'/checkout/cs_old_receipt#token='+'b'*64);ready('Your plan is ready');time.sleep(5)
check('reopened receipt does not redirect',evaluate('location.pathname==="/checkout/cs_old_receipt"'))
scenario('merchant');command('open',BASE+'/checkout/cs_other_merchant#token='+'b'*64);ready('Payment received');time.sleep(5)
check('ordinary merchant stays on its receipt',evaluate('location.pathname==="/checkout/cs_other_merchant"'))
scenario('rejected');checkout('rejected');before=sent();find('button','Pay 5 USDT','click');ready('You declined the payment')
check('rejection is retryable without a send',sent()==before and evaluate('!document.querySelector(".pc-primary").disabled'))
scenario('approval');checkout('approval');before=sent();find('button','Pay 5 USDT','click');ready('Approve token in your wallet');ready('Confirm in your wallet');ready('Confirming payment')
check('approval and payment are separate wallet requests',sent()==before+2)
scenario('failed');ready('The transaction failed')
check('verified failure restores payment action',evaluate('!!document.querySelector(".pc-primary") && !document.querySelector(".pc-primary").disabled'))
scenario('expired');stage('expired');ready('Checkout expired')
check('expired checkout clears stale retry feedback',evaluate('!document.querySelector(".pc-notice")'))
scenario('awaiting');checkout('visual')
for width,height in [(320,900),(390,844),(1440,1000)]:
 command('set','viewport',str(width),str(height))
 check(f'checkout does not overflow at {width}px',evaluate('document.documentElement.scrollWidth <= innerWidth'))
 command('screenshot',str(pathlib.Path(f'target/pay-fullstack-evidence/checkout-{width}.png').resolve()))
find('button','Toggle theme','click')
check('status uses accessible announcements',evaluate('document.querySelector(".pc-steps").getAttribute("aria-label")==="Payment progress"'))
pathlib.Path('target/pay-fullstack-evidence/checks.json').write_text(json.dumps(checks,indent=2))
print(json.dumps(checks,indent=2))
