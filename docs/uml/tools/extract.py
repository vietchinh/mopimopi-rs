import re,os,json,collections
ROOT=os.environ.get('REPO','.')+'/src'
files={}
for d,_,fs in os.walk(ROOT):
    for f in fs:
        if f.endswith('.rs'):
            p=os.path.join(d,f); files[os.path.relpath(p,ROOT)]=open(p).read()
def modpath(rel):
    p=rel[:-3].split('/')
    if p[-1]=='mod': p=p[:-1]
    return p
mods={tuple(modpath(r)):r for r in files}
ITEM=re.compile(r'^(?P<attrs>(?:#\[[^\n]*\]\n)*)(?P<vis>pub(?:\([a-z]+\))?\s+)?(?P<async>async\s+)?(?P<kind>fn|struct|enum|const|static|trait|type|impl)\b\s*(?P<name>[A-Za-z_<][\w<>:, &\'"+]*)?',re.M)
def split_items(src):
    # top-level items (col 0) and impl methods (4 spaces, inside impl)
    lines=src.split('\n'); items=[]; i=0; cur_impl_trait=False
    starts=[]
    for idx,l in enumerate(lines):
        m=re.match(r'^(pub(?:\([a-z]+\))?\s+)?(async\s+)?(fn|struct|enum|const|static|trait|type|impl|mod)\s+([A-Za-z_][\w]*)?',l)
        mi=re.match(r'^    (pub(?:\([a-z]+\))?\s+)?(async\s+)?fn\s+([A-Za-z_]\w*)',l)
        if m and m.group(3)=='impl': cur_impl_trait=bool(re.match(r'^impl(<[^>]*>)?\s+[\w:<>, \']+\s+for\s',l))
        if mi: mi_trait=cur_impl_trait
        if m: starts.append((idx,'top',m.group(3),m.group(4) or '',bool(m.group(1)),l))
        elif mi: starts.append((idx,'method' if not cur_impl_trait else 'traitmethod',"fn",mi.group(3),bool(mi.group(1)),l))
    for n,(idx,lvl,kind,name,pub,l) in enumerate(starts):
        end=len(lines)
        for (j,lv2,*_) in starts[n+1:]:
            if lv2=='top' or lvl in('method','traitmethod'): end=j;break
        if kind=='impl': 
            # impl header: name is type or trait; keep for methods
            pass
        # doc comments and attrs above
        k=idx-1; doc=[]; attrs=[]
        while k>=0 and (lines[k].startswith('///') or lines[k].lstrip().startswith('#[') or lines[k].startswith('    ///')):
            t=lines[k].strip()
            if t.startswith('///'): doc.insert(0,t[3:].strip())
            else: attrs.insert(0,t)
            k-=1
        sig=l.strip()
        body='\n'.join(lines[idx:end])
        items.append(dict(level=lvl,kind=kind,name=name,pub=pub,doc=' '.join(doc),attrs=attrs,sig=sig,body=body,line=idx+1))
    return items
data={}
for rel,src in files.items():
    hdr=[]
    for l in src.split('\n'):
        if l.startswith('//!'): hdr.append(l[3:].strip())
        elif l.strip()=='' and hdr: hdr.append('')
        elif hdr or not l.startswith('//'): 
            if not l.startswith('//!'): break
    data[rel]=dict(doc='\n'.join(hdr).strip(),items=split_items(src),lines=len(src.split('\n')),src=src)
# definitions: name -> files (pub or crate-visible top-level items; methods separately)
defs=collections.defaultdict(set); meths=collections.defaultdict(set)
for rel,d in data.items():
    for it in d['items']:
        if it['level']=='top' and it['kind'] in('fn','struct','enum','const','static','trait','type') and it['name']:
            defs[it['name']].add(rel)
        if it['level']=='method': meths[it['name']].add(rel)
def layer(rel): return rel.split('/')[0]
# resolve use statements per file -> {leaf: file or module}
def expand(u):
    u=u.strip(); m=re.match(r'^(.*?)\{(.*)\}(.*)$',u,re.S)
    if not m: return [u]
    pre,inner,post=m.groups(); out=[]; depth=0; cur=''
    for ch in inner+',':
        if ch=='{':depth+=1
        if ch=='}':depth-=1
        if ch==',' and depth==0:
            if cur.strip(): out+=expand(pre+cur.strip()+post)
            cur=''
        else: cur+=ch
    return out
def resolve_path(parts,rel):
    cur=modpath(rel); pkg=cur if rel.endswith('mod.rs') else cur[:-1]
    if parts[0]=='crate': parts=parts[1:]
    elif parts[0]=='super':
        base=list(pkg)
        while parts and parts[0]=='super': base=base[:-1]; parts=parts[1:]
        parts=base+parts
    elif parts[0]=='self': parts=list(pkg)+parts[1:]
    else: return None,None
    for n in range(len(parts),0,-1):
        t=tuple(parts[:n])
        if t in mods: return mods[t],(parts[n:] or [None])
    return None,None
imports={}
for rel,d in data.items():
    imp={}
    for m in re.finditer(r'^\s*(?:pub(?:\([a-z]+\))?\s+)?use\s+([^;]+);',d['src'],re.M):
        for u in expand(m.group(1)):
            alias=None
            mm=re.match(r'^(.*)\s+as\s+(\w+)$',u)
            if mm: u,alias=mm.groups()
            parts=[p for p in u.replace('::*','').split('::') if p]
            glob=u.endswith('::*')
            if not parts or parts[0] not in('crate','super','self'): continue
            f,rest=resolve_path(parts,rel)
            leaf=alias or parts[-1]
            imp[leaf]=(parts,f,glob)
    imports[rel]=imp
# pub names re-exported through mod.rs: map symbol -> real defining file by name
def target_of(sym,rel,imp_entry):
    parts,f,glob=imp_entry
    cands=[x for x in defs.get(parts[-1],[]) if x!=rel]
    if parts[-1] in ('default','new','from','into'): return None
    if len(cands)==1: return cands[0]
    if not cands and tuple(modpath(f or ''))[-1:]!=(parts[-1],): return None
    if f and f!=rel:
        # prefer a defining file within the resolved package
        pk=os.path.dirname(f) if f.endswith('mod.rs') else None
        inpk=[c for c in cands if pk and c.startswith(pk)]
        if len(inpk)==1: return inpk[0]
        return f
    return None
for rel,d in data.items():
    imp=imports[rel]
    for it in d['items']:
        b=it['body']; calls=collections.OrderedDict(); renders=collections.OrderedDict()
        for leaf,e in imp.items():
            if re.search(r'(?<![\w:])'+re.escape(leaf)+r'\b',b) and not b.startswith('use'):
                t=target_of(leaf,rel,e)
                if t: calls.setdefault(t,set()).add(leaf)
        # qualified paths
        for m in re.finditer(r'\b((?:crate|super)(?:::\w+)+)',b):
            parts=m.group(1).split('::'); f,rest=resolve_path(parts,rel)
            sym=parts[-1]
            c=[x for x in defs.get(sym,[]) if x!=rel]
            t=c[0] if len(c)==1 else (f if f and f!=rel else None)
            if t: calls.setdefault(t,set()).add(sym)
        # components rendered
        for m in re.finditer(r'\b([A-Z][A-Za-z0-9]+)\s*\{',b):
            n=m.group(1)
            for f in defs.get(n,[]):
                if f!=rel and any(a=='#[component]' for x in data[f]['items'] if x['name']==n for a in x['attrs']):
                    renders.setdefault(f,set()).add(n)
        # methods
        for m in re.finditer(r'\.([a-z_][a-z0-9_]{5,})\(',b):
            n=m.group(1)
            fs_=meths.get(n,set())
            if len(fs_)==1 and list(fs_)[0]!=rel: calls.setdefault(list(fs_)[0],set()).add('.'+n+'()')
        ctx=[]
        for m in re.finditer(r'\b(use_context|consume_context|try_consume_context|use_context_provider)(?:::<([\w:]+)>)?',b):
            kind,ty=m.group(1),m.group(2)
            if ty:
                ty=ty.split('::')[-1]
                for f in defs.get(ty,[]):
                    if f!=rel: ctx.append((kind,ty,f))
        it['contexts']=sorted(set(ctx))
        fields=sorted(set(re.findall(r'\b(?:view|settings_view)\.(bars|columns|nav|page|raid|table)\.(?:read|peek)\(',b)))
        it['area_reads']=fields
        sg=b.split('{')[0] if it['kind']=='fn' else it['sig']
        mret=re.search(r'->\s*(.+?)\s*$',sg.replace('\n',' '),re.S)
        it['ret']=mret.group(1).strip() if mret else ''
        it['calls']={k:sorted(v) for k,v in calls.items()}
        it['renders']={k:sorted(v) for k,v in renders.items()}
        for k in ('body',): pass
out={}
for rel,d in data.items():
    if not rel.startswith('presentation/'): continue
    out[rel]=dict(doc=d['doc'],lines=d['lines'],items=[{k:v for k,v in it.items() if k!='body'} for it in d['items']],imports={k:v[0] for k,v in imports[rel].items()})
json.dump(out,open('facts.json','w'),indent=1)
print(len(out),sum(len(v['items']) for v in out.values()))
