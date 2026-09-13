#!/usr/bin/env python3
"""Control only com.epsx.dev LaunchAgents. Config and keys stay outside releases."""
import argparse, os, pathlib, plistlib, shutil, subprocess, time, urllib.request, urllib.error, fcntl
ROOT = pathlib.Path.home() / '.config/epsx/dev'
RELEASES = pathlib.Path.home() / '.local/share/epsx/dev'
SERVICES = ['epsx','wallet','pay-service','subscription','notification','analytics','bff-frontend','bff-admin','bff-pay']
UI = ['bff-frontend', 'bff-admin', 'bff-pay']
PORTS = dict(zip(UI, [3000, 3001, 3002]))
CHECKOUT = pathlib.Path(__file__).resolve().parents[2]
ALL = ['minio','anvil',*SERVICES,'tunnel']
DOMAIN = f'gui/{os.getuid()}'

def job(name): return f'com.epsx.dev.{name}'
def path(name): return pathlib.Path.home()/'Library/LaunchAgents'/(job(name)+'.plist')
def run(*args, check=True): return subprocess.run(args, check=check)
def install(name):
    root = str(ROOT)
    if name == 'minio':
        args = ['/bin/bash',root+'/tools/minio.sh']
    elif name == 'anvil':
        shutil.copy2(pathlib.Path(__file__).with_name('dev-anvil.py'), ROOT/'tools/dev-anvil.py')
        args = ['/usr/bin/python3',root+'/tools/dev-anvil.py']
    elif name == 'tunnel':
        args = ['/opt/homebrew/bin/cloudflared','tunnel','--config',root+'/config/tunnel.yml','run','epsx-dev']
    else:
        args = ['/bin/bash',str(RELEASES/'current/ops/run-service.sh'),name,root+'/config']
    plist={'Label':job(name),'ProgramArguments':args,'WorkingDirectory':root,'RunAtLoad':True,'KeepAlive':True,'ThrottleInterval':10,'ExitTimeOut':30,'StandardOutPath':root+'/logs/'+name+'.log','StandardErrorPath':root+'/logs/'+name+'.log','EnvironmentVariables':{'PATH':'/opt/homebrew/bin:/usr/bin:/bin'},'ProcessType':'Background'}
    p=path(name);p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(plistlib.dumps(plist));p.chmod(0o600)

def loaded(name):
    return subprocess.run(['launchctl', 'print', DOMAIN+'/'+job(name)],
                          capture_output=True).returncode == 0

def hmr_plist(name):
    p = path(name)
    if p.exists():
        config = plistlib.loads(p.read_bytes())
    else:
        config = {'Label': job(name), 'RunAtLoad': True, 'KeepAlive': True,
                  'ThrottleInterval': 10, 'ExitTimeOut': 30, 'ProcessType': 'Background',
                  'StandardOutPath': str(ROOT/'logs'/f'{name}.log'),
                  'StandardErrorPath': str(ROOT/'logs'/f'{name}.log')}
    config['ProgramArguments'] = ['/bin/bash', str(CHECKOUT/'infrastructure/native/dev-ui-hmr.sh'), name]
    config['WorkingDirectory'] = str(CHECKOUT)
    return config

def enable_hmr(name):
    if name == 'ui-worker':
        (ROOT/'tools').mkdir(parents=True, exist_ok=True)
        shutil.copy2(CHECKOUT/'infrastructure/native/dev-ui-assets.py', ROOT/'tools/dev-ui-assets.py')
    p = path(name)
    config = hmr_plist(name)
    if p.exists() and plistlib.loads(p.read_bytes()).get('ProgramArguments') == config['ProgramArguments']:
        if not loaded(name): run('launchctl', 'bootstrap', DOMAIN, str(p))
        return
    backup = p.with_suffix('.plist.pre-hmr')
    if p.exists() and not backup.exists(): shutil.copy2(p, backup)
    if loaded(name): run('launchctl', 'bootout', DOMAIN, str(p))
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(plistlib.dumps(config)); p.chmod(0o600)
    run('launchctl', 'bootstrap', DOMAIN, str(p))

def wait_ui(name, timeout=1800):
    deadline = time.monotonic() + timeout
    print(f'{name}: waiting for DX build and HTTP on :{PORTS[name]}', flush=True)
    while time.monotonic() < deadline:
        try:
            request = urllib.request.Request(f'http://127.0.0.1:{PORTS[name]}/', headers={'Accept':'text/html'})
            with urllib.request.urlopen(request, timeout=5) as response:
                body = response.read().decode(errors='replace')
                if response.status == 200 and 'id="main"' in body:
                    print(f'{name}: ready (HTTP 200)', flush=True)
                    return
        except (OSError, urllib.error.URLError): pass
        time.sleep(2)
    raise SystemExit(f'{name}: not ready; see {ROOT}/logs/{name}.log. Remaining UIs were not started.')

def hmr(names):
    # Serialize invocations, reuse LaunchAgents, and build one UI at a time.
    ROOT.mkdir(parents=True, exist_ok=True)
    with (ROOT/'hmr.lock').open('w') as lock:
        try: fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError: raise SystemExit('Another HMR startup is already running.')
        env = dict(os.environ, CARGO_TARGET_DIR=str(CHECKOUT/'target'), CARGO_BUILD_JOBS='2', CARGO_INCREMENTAL='0')
        subprocess.run(['cargo','xtask','browser-runtime','build'], cwd=CHECKOUT, env=env, check=True)
        enable_hmr('ui-worker')
        for name in names:
            enable_hmr(name)
            wait_ui(name)

def restore_hmr(names):
    for name in names:
        p = path(name); backup = p.with_suffix('.plist.pre-hmr')
        if not backup.exists(): raise SystemExit(f'No pre-HMR backup for {name}')
    for name in names:
        p = path(name); backup = p.with_suffix('.plist.pre-hmr')
        if loaded(name): run('launchctl','bootout',DOMAIN,str(p))
        shutil.copy2(backup,p)
        run('launchctl','bootstrap',DOMAIN,str(p))
    if not any(path(n).exists() and 'dev-ui-hmr.sh' in str(plistlib.loads(path(n).read_bytes()).get('ProgramArguments')) for n in UI):
        if loaded('ui-worker'): run('launchctl','bootout',DOMAIN,str(path('ui-worker')))
        path('ui-worker').unlink(missing_ok=True)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('action',choices=['install','start','stop','restart','status','rollback','watch','hmr','restore-hmr'])
    p.add_argument('service',nargs='?',choices=ALL+['all','ui','ui-worker'],default='all')
    a=p.parse_args();names=ALL if a.service=='all' else UI if a.service=='ui' else [a.service]
    if a.action in ('hmr','restore-hmr'):
        if a.service not in UI+['ui']: p.error('HMR requires ui or a bff UI service')
        (hmr if a.action=='hmr' else restore_hmr)(names)
        return
    if a.action=='watch':
        if a.service != 'bff-frontend':raise SystemExit('watch currently supports bff-frontend only')
        n=a.service
        existing=path(n)
        backup=existing.with_suffix('.plist.release-backup')
        if not backup.exists():shutil.copy2(existing,backup)
        plist=plistlib.loads(existing.read_bytes())
        checkout=pathlib.Path(__file__).resolve().parents[2]
        plist['ProgramArguments']=['/bin/bash',str(checkout/'infrastructure/native/dev-frontend-watch.sh')]
        plist['WorkingDirectory']=str(checkout)
        run('launchctl','bootout',DOMAIN,str(existing),check=False)
        existing.write_bytes(plistlib.dumps(plist));existing.chmod(0o600)
        run('launchctl','bootstrap',DOMAIN,str(existing))
        return
    if a.action=='rollback':
        previous=RELEASES/'previous';current=RELEASES/'current'
        if not previous.is_symlink() or not previous.resolve().is_dir():raise SystemExit('No previous compatible dev release')
        old=current.resolve();new=previous.resolve()
        snapshot=ROOT/'config-snapshots'/new.name
        if not snapshot.is_dir():raise SystemExit('Previous dev configuration snapshot is missing')
        names=SERVICES
        for n in reversed(names):run('launchctl','bootout',DOMAIN,str(path(n)),check=False)
        current_snapshot=ROOT/'config-snapshots'/old.name
        current_snapshot.mkdir(parents=True,exist_ok=True,mode=0o700)
        for filename in ['common.env',*[n+'.env' for n in SERVICES]]:
            shutil.copy2(ROOT/'config'/filename,current_snapshot/filename)
            shutil.copy2(snapshot/filename,ROOT/'config'/filename)
        temp=RELEASES/'next'
        temp.symlink_to(new);os.replace(temp,current);previous.unlink();previous.symlink_to(old)
        for n in names:run('launchctl','bootstrap',DOMAIN,str(path(n)))
        return
    for n in (reversed(names) if a.action=='stop' else names):
        if a.action=='install':install(n)
        elif a.action=='start':run('launchctl','bootstrap',DOMAIN,str(path(n)))
        elif a.action=='stop':run('launchctl','bootout',DOMAIN,str(path(n)),check=False)
        elif a.action=='restart':
            # Graceful stop is essential to persist Anvil's latest chain state.
            run('launchctl','bootout',DOMAIN,str(path(n)),check=False)
            run('launchctl','bootstrap',DOMAIN,str(path(n)))
        elif a.action=='status':
            r=subprocess.run(['launchctl','print',DOMAIN+'/'+job(n)],capture_output=True,text=True)
            state=[x.strip() for x in r.stdout.splitlines() if x.strip().startswith(('state =','pid =','last exit code ='))]
            print(n,', '.join(state) if r.returncode==0 else 'not loaded')
if __name__=='__main__':main()
