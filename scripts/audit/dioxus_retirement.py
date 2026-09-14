"""Structural retirement checks shared by inventory and its regression tests.
Only actual #[cfg(test)] Rust items are excluded; code after a test item stays live.
"""
import re
from pathlib import Path

LEX = re.compile(r'r(?P<hash>#{0,32})".*?"(?P=hash)|"(?:\\.|[^"\\])*"|//[^\n]*|/\*.*?\*/|\'(?:\\.|[^\'\\])\'', re.S)
def code_mask(source):
    return LEX.sub(lambda m: ''.join('\n' if c=='\n' else ' ' for c in m.group()),source)

def test_ranges(source):
    code=code_mask(source); spans=[]
    for m in re.finditer(r'#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]',code):
        start=m.start(); i=m.end(); depth=0; opened=False
        statement=bool(re.match(r'\s*let\b',code[i:]))
        while i<len(code):
            c=code[i]
            if c=='{':depth+=1;opened=True
            elif c=='}':
                depth-=1
                if opened and depth==0 and not statement:
                    i+=1;break
            elif c==';' and depth==0: i+=1;break
            i+=1
        spans.append((start,i))
    return spans

def production_source(source):
    chars=list(source)
    for a,b in test_ranges(source):
        chars[a:b]=['\n' if c=='\n' else ' ' for c in source[a:b]]
    return ''.join(chars)

def test_module_files(root):
    result=set()
    for path in root.rglob('*.rs'):
        source=path.read_text()
        for a,b in test_ranges(source):
            declaration=re.search(r'\bmod\s+(\w+)\s*;',code_mask(source[a:b]))
            if declaration:
                name=declaration[1]; parent=path.parent if path.name in ('mod.rs','lib.rs','main.rs') else path.with_suffix('')
                file=parent/(name+'.rs'); directory=parent/name
                if file.is_file():result.add(file)
                if directory.is_dir():result.update(directory.rglob('*.rs'))
    return result

def boundary_errors(root):
    errors=[]
    sources={app:production_source((root/f'apps/{app}/src/lib.rs').read_text()) for app in ('frontend','admin','pay')}
    forbidden=r'\b(?:ssr_handler|page_html|browser_runtime_router)\s*\(|\bdioxus_ssr::|\b(?:native_escrow::(?:page|merchant_page)|plan_catalog::(?:catalog|orders)|pay_orders::page)\b'
    for app,source in sources.items():
        for m in re.finditer(forbidden,code_mask(source)):
            errors.append(f'apps/{app}/src/lib.rs:{source.count(chr(10),0,m.start())+1}: retired native UI entrypoint {m.group()}')
    if not re.search(r'pub fn build_app\([^)]*\)\s*->\s*Router\s*\{\s*fullstack::application\(state\)\s*\}',sources['frontend']):
        errors.append('Frontend build_app must delegate directly to Fullstack application')
    for app in ('frontend','admin','pay'):
        source=production_source((root/f'apps/{app}/src/fullstack.rs').read_text())
        if 'FullstackState' not in source:errors.append(f'{app} has no Fullstack state')
        for token in ('epsx_browser_runtime','page_shell_with_body_class','dioxus_ssr::render'):
            if token in code_mask(source):errors.append(f'{app} Fullstack includes retired producer {token}')
    for app,component in [('frontend','FrontendRoot'),('admin','AdminRoot'),('pay','PayApp')]:
        web=production_source((root/f'apps/{app}/src/dx_main.rs').read_text())
        native=production_source((root/f'apps/{app}/src/fullstack.rs').read_text())
        if not re.search(r'dioxus::launch\([^;]*\b'+component+r'\)',web) or not re.search(r'FullstackState::new\([\s\S]*?\b'+component+r'\s*[,)]',native):
            errors.append(f'{app} SSR and hydration must launch the same {component} tree')
    # Route trees cannot opt back into PageContext/static render wrappers.
    routes=production_source((root/'shared/rust/dioxus_ui/src/routes.rs').read_text())
    for token in ('PageContext','render_page(','render_dynamic(','dangerous_inner_html'):
        if token in code_mask(routes):errors.append(f'Router reintroduced legacy page owner: {token}')
    return errors
