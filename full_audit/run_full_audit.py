from __future__ import annotations
import argparse, json, math, re, statistics, sys, tempfile, time
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'golden'))
from mcp_client import StdioMcpClient, mcp_text
try:
    import tiktoken
except Exception:
    tiktoken=None

EXPECTED_OPS=1215

def registry_ops():
    """Load the frozen alpha.12 opcode manifest.

    Runtime truth is verified immediately afterward by querying ``yk.spec`` for
    every manifest opcode.  We intentionally do not re-parse ``registry.rs``
    here: the source-text parser proved platform-sensitive on Windows and could
    truncate the audit scope.  A manifest mismatch therefore becomes a live
    ``yk.spec`` coverage failure instead of a misleading preflight count.
    """
    manifest_path=ROOT/'full_audit/opcodes_alpha12.json'
    manifest=json.loads(manifest_path.read_text(encoding='utf-8-sig'))
    declared=manifest.get('opcodes',[])
    declared_count=manifest.get('count')
    if declared_count != EXPECTED_OPS or len(declared) != EXPECTED_OPS:
        raise RuntimeError(f'full-audit opcode manifest corrupt: count={declared_count}, entries={len(declared)}, expected={EXPECTED_OPS}')
    if len(set(declared)) != EXPECTED_OPS:
        raise RuntimeError('full-audit opcode manifest contains duplicates')
    if not all(isinstance(op,str) and op and '.' in op for op in declared):
        raise RuntimeError('full-audit opcode manifest contains invalid opcode entries')
    return declared

def load_primary_fixtures():
    cases=json.loads((ROOT/'golden/cases.json').read_text(encoding='utf-8-sig'))['cases']
    out={}
    for c in cases:
        if 'expect' in c and c['op'] not in out:
            out[c['op']]=c['a']
    for op,a in json.loads((ROOT/'full_audit/overrides_alpha12.json').read_text(encoding='utf-8')).items(): out.setdefault(op,a)
    out['expr.eval']=[{'e':'1+2'}]
    return out

def load_family_fixture_corpus():
    """Reusable successful inputs from the Golden corpus and curated overrides.

    Legacy families often expose ``args...`` in yk.spec, so the audit cannot infer
    arity from the signature alone.  Reusing already-proven sibling inputs gives
    fixture discovery a semantically grounded fallback without changing runtime
    code or pretending those sibling inputs are correctness oracles for the new op.
    """
    by_family=defaultdict(list)
    seen=defaultdict(set)
    cases=json.loads((ROOT/'golden/cases.json').read_text(encoding='utf-8-sig'))['cases']
    items=[]
    for c in cases:
        if 'expect' in c:
            items.append((c['op'],c['a']))
    items.extend(json.loads((ROOT/'full_audit/overrides_alpha12.json').read_text(encoding='utf-8')).items())
    for op,a in items:
        fam=op.split('.',1)[0]
        key=json.dumps(a,sort_keys=True,separators=(',',':'))
        if key not in seen[fam]:
            seen[fam].add(key); by_family[fam].append(a)
    return by_family

FAMILY_CORPUS=load_family_fixture_corpus()


SEMANTIC_FIXTURES={
    # final targeted fixtures for operations whose valid domains depend on
    # shape, integer arity, bounded coefficients, or argument relationships.
    'mat.reshape': [[1.0,2.0,3.0,4.0],2,2],
    'prob.normalize_weights': [[1.0,2.0,3.0]],
    'net.subnet_count': [24,26],
    # color vectors with fixed dimensionality; alpha is normalized to [0,1].
    'color.alpha_over': [[255.0,0.0,0.0,0.5],[0.0,0.0,255.0,1.0]],
    'color.contrast_ratio': [[0.0,0.0,0.0],[255.0,255.0,255.0]],
    'color.premultiply': [[128.0,64.0,32.0],0.5],
    'color.relative_luminance': [[128.0,64.0,32.0]],
    'color.unpremultiply': [[64.0,32.0,16.0,0.5]],
    'color.ycbcr_to_rgb': [[128,128,128]],
    'color.yiq_to_rgb': [[0.5,0.1,0.1]],
    'color.srgb_to_linear': [[128,64,32]],
    'color.linear_to_srgb': [[0.2,0.4,0.6]],
    # domain relationships that generic scalar synthesis cannot infer.
    'astro.equilibrium_temperature': [5800.0,6.96e8,1.496e11,0.3],
    'thermo.radiation': [0.9,2.0,400.0,300.0],
    'thermo.stefan_flux': [0.9,400.0],
    'wave.doppler_source': [440.0,343.0,10.0],
    'wave.doppler_full': [440.0,343.0,5.0,10.0],
    'frame.rot_x': [0.5,'a','b'],
    'frame.rot_y': [0.5,'a','b'],
    'frame.rot_z': [0.5,'a','b'],
    # frame composition must be chain-compatible
    'frame.compose': [
        {'from':'a','to':'b','r':[[1,0,0],[0,1,0],[0,0,1]],'p':[1,0,0]},
        {'from':'b','to':'c','r':[[1,0,0],[0,1,0],[0,0,1]],'p':[0,1,0]},
    ],
    'frame.same': [
        {'f':'a','t':'v','v':[1,0,0]},
        {'f':'a','t':'v','v':[0,1,0]},
    ],
    'frame.add_vec': [
        {'f':'a','t':'v','v':[1,0,0]},
        {'f':'a','t':'v','v':[0,1,0]},
    ],
    'frame.sub_vec': [
        {'f':'a','t':'v','v':[1,1,0]},
        {'f':'a','t':'v','v':[0,1,0]},
    ],
    'frame.point_plus_vec': [
        {'f':'a','t':'p','v':[1,2,3]},
        {'f':'a','t':'v','v':[1,0,0]},
    ],
    'frame.point_minus_point': [
        {'f':'a','t':'p','v':[2,2,3]},
        {'f':'a','t':'p','v':[1,2,3]},
    ],
    # geometry predicates need nested/ordered shapes rather than scalar placeholders
    'predicate.circle_in_circle': [[0,0],1,[0,0],3],
    'predicate.circles_intersect': [[0,0],1,[1,0],1],
    'predicate.aabb_contains': [[[0,0],[4,4]],[[1,1],[2,2]]],
    'predicate.point_in_aabb': [[1,1],[[0,0],[2,2]]],
    'predicate.polygon_in_polygon': [
        [[1,1],[2,1],[2,2],[1,2],[1,1]],
        [[0,0],[4,0],[4,4],[0,4],[0,0]],
    ],
    # thermodynamic ordering constraints
    'thermo.carnot_efficiency': [400.0,300.0],
    'thermo.carnot_cop_refrigerator': [400.0,300.0],
    'thermo.carnot_cop_heatpump': [400.0,300.0],
    # network/string contracts
    'net.same_subnet': ['192.168.1.10','192.168.1.20',24],
    'net.host_offset': ['192.168.1.0/24','192.168.1.10'],
    'net.subnet_index': ['192.168.1.10',24],
    'net.supernet': ['192.168.1.0/24',16],
    'net.cidr_normalize': ['192.168.1.10/24'],
    # series objects are structured payloads
    'series.fourier_eval': [{'a':[0.0,1.0],'b':[0.0,0.0],'period':6.283185307179586},0.5],
    'series.chebyshev_eval': [{'c':[0.5,0.0,0.5],'lo':-1.0,'hi':1.0},0.3],
    'series.power_eval': [[1.0,2.0,3.0],0.5],
    'series.taylor_eval': [[1.0,2.0,3.0],0.5],
    # ODE method code is an enum-like integer, not an arbitrary float
    'ode.step_doubling_error': [{'e':'y'},0.0,1.0,0.1,4],
}

def _numeric_tuples():
    out=[]
    for n in range(1,7):
        out += [
            [2.0]*n,
            [1.0]*n,
            [0.5]*n,
            [float(i+1) for i in range(n)],
        ]
    return out

NUMERIC_TUPLES=_numeric_tuples()

LEGACY_BANK={
'prob':[
    [0.5],[1,0.5],[2,0.5],[3,0.5],[5,2,0.5],[5,2,10],
    [0.2,0.5],[0.2,0.3,0.5],[0,1,-1,1],[0,1],
    [[1,2,3],[0.2,0.3,0.5]],[[0.2,0.3,0.5]],
    [[1,2,3],[2,4,6],[0.2,0.3,0.5]],
],
'alg':[
    [5],[10],[10,3],[10,3,7],[2,3,3,5],[1,2,5],
    ['12'],['12','18'],['17','5'],[[1,2,3]],[[1,2,3],2],
    [[1,2,3],[1,1]],[[1,0,-1]],
],
'cplx':[
    [[1,2]],[[1,2],[3,4]],[[1,2],2],[[1,2],0.5],[[1,2],2,0.5],
    [[1,2],[3,4],0.5],[[1,0],2],[[1,0],3],
],
'mat':[
    [[[2,0],[0,1]]],
    [[[2,0],[0,1]],[[1,0],[0,2]]],
    [[[2,0],[0,1]],[1,2]],
    [[[2,0],[0,1]],0],
    [[[2,0],[0,1]],1],
    [[[2,0],[0,1]],0,1],
    [[[2,0],[0,1]],2],
    [[[1,2],[3,4]],2,2],
    [[[1,2],[3,4]],[5,6]],
],
'stat':[
    [[1,2,3,4]],[[1,2,3,4],[2,4,6,8]],[[1,2,3,4],0.5],
    [[1,2,3,4],25.0],[[1,2,3,4],1],[[1,2,3,4],2],
    [[1,1,2,2,3,3]],[[1,2,3,100],1.5],
],
'signal':[
    [[1,2,3,4]],[[1,2,3,4],2],[[1,2,3,4],0.5],
    [[1,2,3,4],[1,1,1,1]],[[1,2,3,4],8],[[1,2,3,4],0,1],
],
'num':[
    [{'e':'x^2-2'},1.0,2.0],[{'e':'x^2'},1.0],[{'e':'x'},0.0,1.0],
    [{'e':'x^2'},0.0,1.0,100],[{'e':'x^2'},1.0,1e-4],
    [[1,2,3,4]],[[1,2,3,4],1.0],[[0,1,2],[0,1,4],1.5],
    [0.0,1.0,5],[1.0,10.0,5],
],
'reg':[
    [[1,2,3],[2,4,6]],[[1,2,3],[2,4,6],2.0],
    [[1,2,3],[2,4,6],[1,1,1]],[[1,2,3],[2,4,6],[2,3,4]],
],
'test':[
    [[1,2,3,4],2.0],[[1,2,3],[2,3,4]],[[10,20],[12,18]],
    [10,5,0.5],[10,5,12,6],[0.5,100,0.4,100],
    [[10,20,30],[15,25,35]],
],
'eng':NUMERIC_TUPLES,
'phys':NUMERIC_TUPLES,
}

def val_for(token:str,op:str,index:int):
    t=token.rstrip('?').strip().lower()
    fam=op.split('.',1)[0]
    if t in {'expr','{e,v?}'}: return {'e':'x^2+1'}
    if t=='dy_expr': return {'e':'y'}
    if 'matrix' in t or t in {'a_matrix','b_matrix'}: return [[2,0],[0,1]]
    if 'aabb' in t: return [[0,0],[2,2]]
    if 'polygon' in t or t=='points': return [[0,0],[2,0],[2,2],[0,2],[0,0]]
    if 'point3' in t or t=='xyz': return [1,2,3]
    if 'point' in t or 'center' in t: return [1,1]
    if 'vector' in t: return [1,0,0]
    if 'rgba' in t: return [128,64,32,255]
    if 'rgb' in t: return [128,64,32]
    if 'linear_rgb' in t: return [0.2,0.4,0.6]
    if 'ycbcr' in t: return [128,128,128]
    if 'yiq' in t: return [0.5,0.1,0.1]
    if 'hsv' in t: return [120,0.5,0.5]
    if 'hsl' in t: return [120,0.5,0.5]
    if 'cmyk' in t: return [0.1,0.2,0.3,0.1]
    if 'probabilit' in t or t in {'p[]','weights[]'}: return [0.2,0.3,0.5]
    if 'number[]' in t or 'values[]' in t or 'residuals[]' in t or 'samples'==t or 'array'==t or 'terms'==t or 'sequence'==t or 'coefficients'==t or 'partial_sums'==t or 'uniform_samples'==t: return [1,2,3]
    if 'x[]' in t: return [1,2,3]
    if 'y[]' in t: return [2,4,6]
    if 'ipv4' in t: return '192.168.1.10'
    if 'cidr' in t: return '192.168.1.0/24'
    if 'hex' in t: return '#336699'
    if 'decimal|string' in t: return '12.5'
    if 'integer|string' in t: return '10'
    if t in {'string','digits'}: return '1010'
    if t in {'from_base'}: return 10
    if t in {'to_base'}: return 16
    if t=='frame': return 'a'
    if t in {'transform','transform_ab'}: return {'from':'a','to':'b','r':[[1,0,0],[0,1,0],[0,0,1]],'p':[0,0,0]}
    if t=='transform_bc': return {'from':'b','to':'c','r':[[1,0,0],[0,1,0],[0,0,1]],'p':[0,0,0]}
    if t=='tagged': return {'f':'a','t':'v','v':[1,0,0]}
    if t in {'outer','inner'} and fam=='predicate': return [[0,0],[4,4]] if t=='outer' else [[1,1],[2,2]]
    if t=='coeff_object' and op=='series.fourier_eval': return {'a':[0.0,1.0],'b':[0.0,0.0],'period':6.283185307179586}
    if t=='coeff_object' and op=='series.chebyshev_eval': return {'c':[0.5,0.0,0.5],'lo':-1.0,'hi':1.0}
    if t in {'from','to'} and fam=='unit': return 'm' if index==1 else 'cm'
    if t in {'from','to'} and fam=='color': return 'rgb' if index==0 else 'hsv'
    if 'op,p,expr' in t: return {'op':'user.audit_formula','p':['x'],'expr':'x+1'}
    if 'op,p,pipe' in t: return {'op':'user.audit_composite','p':['x'],'pipe':[{'op':'math.add','a':['$a0',1]}]}
    if 'user|pack opcode' in t: return 'user.audit_missing'
    if 'name,ops' in t: return {'name':'audit','ops':['user.audit_formula']}
    if t=='pack name': return 'auditpack'
    if t=='pack': return {'v':1,'name':'auditpack','ops':[{'k':'f','op':'pack.auditpack.inc','p':['x'],'expr':'x+1'}]}
    # common domains that must stay positive/nonzero
    positive_words={'radius','mass','density','volume','length','distance','area','time','seconds','minutes','hours','days','weeks','frequency','wavelength','pressure','temperature','temp','kelvin','resistance','capacitance','inductance','viscosity','conductivity','power','energy','speed','velocity','gravity','molar','moles','bit','byte','bandwidth','rate','principal','period','count','n','k','order','steps','samples','prefix','mtu','header','payload','modulus','base_steps','levels','year','month','diameter','height','width','force','charge','current','voltage','focal'}
    parts=set(re.findall(r'[a-z0-9_]+',t))
    if t in positive_words or parts.intersection(positive_words):
        if t=='prefix': return 24
        if t=='month': return 2
        if t=='year': return 2024
        if 'percent' in t: return 50.0
        return 2.0
    if t in {'lo','min'}: return 0.0
    if t in {'hi','max'}: return 4.0
    if t in {'x','y','a','b','c','value','mean','std','lambda','mu','h','t','r1','r2','v1','v2','c1','c2','p1','p2','z1','z2','f1','f2'}: return float(index+1)
    if t.endswith('[]'): return [1,2,3]
    return 2.0

def signature_candidate(op,argspec):
    if argspec==['args...'] or 'args...' in argspec: return None
    vals=[]
    for i,t in enumerate(argspec):
        if t.endswith('?'): break
        vals.append(val_for(t,op,i))
    return vals

GENERIC_BANK={
'math':[[2.0],[0.5],[2.0,3.0],[2.0,3.0,4.0],[[1,2,3]],[[1,2,3],[4,5,6]]],
'stat':[[[1,2,3]],[[1,2,3],[1,1,1]],[[1,2,3],[2,4,6]],[[1,2,3],0.5]],
'alg':[[5],[10,3],['12'],['12','18'],[[1,2,3]],[[1,2,3],[3,4,5]]],
'cplx':[[[1,2]],[[1,2],[3,4]],[1,2],[2.0]],
'prob':[[5,2],[3,0.5],[0,0,1],[[1,2,3],[0.2,0.3,0.5]],[[1,2,3]]],
'mat':[[[[1,0],[0,2]]],[[[1,0],[0,2]],[[2,0],[0,1]]],[[[1,0],[0,2]],[1,2]]],
'vec':[[[1,2,3]],[[1,0,0],[0,1,0]],[[1,2,3],2.0]],
'geo':[[2.0],[2.0,3.0],[[0,0],[1,1]],[[0,0],[1,0],[0,1]]],
'num':[[{'e':'x^2-2'},1.0,2.0],[{'e':'x^2'},1.0],[1.0,2.0,3.0],[[1,2,3]],[[1,2,3],1.0]],
'signal':[[[1,2,3]],[[1,2,3],2],[[1,2,3],[1,1,1]]],
'int':[['10'],['10','3'],['10',2]],'dec':[['1.5'],['1.5','2.5']],
}
DEFAULT_BANK=[[],[2.0],[1.0],[2.0,3.0],[1.0,2.0,3.0],[[1,2,3]],[[1,2,3],[2,4,6]],[[[1,0],[0,1]]],[[1,0,0],[0,1,0]]]

def candidates(op,argspec,primary):
    seen=set()
    def add(a):
        if a is None:return
        key=json.dumps(a,sort_keys=True,separators=(',',':'))
        if key not in seen:
            seen.add(key); yield a
    if op in primary:
        yield from add(primary[op])
    if op in SEMANTIC_FIXTURES:
        yield from add(SEMANTIC_FIXTURES[op])
    c=signature_candidate(op,argspec)
    yield from add(c)
    fam=op.split('.',1)[0]
    # Hand-shaped family candidates first: cheap and deliberately diverse.
    for a in GENERIC_BANK.get(fam,DEFAULT_BANK): yield from add(a)
    for a in LEGACY_BANK.get(fam,[]): yield from add(a)
    # Reuse inputs that are already known to execute successfully for sibling
    # operations in the same family.  This is fixture synthesis only; the
    # Golden oracle is still run separately for mathematical correctness.
    for a in FAMILY_CORPUS.get(fam,[]): yield from add(a)
    # Last-resort numeric arity sweep is useful for legacy ``args...`` specs in
    # scalar engineering/physics/domain packs.
    if argspec==['args...'] or 'args...' in argspec:
        for a in NUMERIC_TUPLES: yield from add(a)
    for a in DEFAULT_BANK: yield from add(a)

def parse_payload(sample):
    text=mcp_text(sample.response)
    try:return json.loads(text)
    except Exception:return {'e':'PARSE','raw':text}

def wire_tokens(sample,enc):
    s=json.dumps(sample.request,separators=(',',':'),ensure_ascii=False)+json.dumps(sample.response,separators=(',',':'),ensure_ascii=False)
    return len(enc.encode(s)) if enc else None

def is_number(v):
    return isinstance(v,(int,float)) and not isinstance(v,bool) and math.isfinite(float(v))

def type_ok(v, spec):
    t=(spec or 'value').strip().lower()
    if t=='value': return True
    if t in {'number'}: return is_number(v)
    if t in {'integer','u64'}: return isinstance(v,int) and not isinstance(v,bool) and (t!='u64' or v>=0)
    if t=='boolean': return isinstance(v,bool)
    if t in {'string','opcode'}: return isinstance(v,str)
    if t=='number|null': return v is None or is_number(v)
    if t=='object' or t=='pack': return isinstance(v,dict)
    if t=='matrix': return isinstance(v,list) and all(isinstance(r,list) and all(is_number(x) for x in r) for r in v)
    if t in {'array','value[]'}: return isinstance(v,list)
    if t=='opcode[]': return isinstance(v,list) and all(isinstance(x,str) for x in v)
    if t=='number[]': return isinstance(v,list) and all(is_number(x) for x in v)
    if t=='complex[]': return isinstance(v,list) and all(isinstance(x,list) and len(x)==2 and all(is_number(y) for y in x) for x in v)
    m=re.fullmatch(r'(number|integer)\[(\d+)\]',t)
    if m:
        if not isinstance(v,list) or len(v)!=int(m.group(2)): return False
        return all(is_number(x) if m.group(1)=='number' else isinstance(x,int) and not isinstance(x,bool) for x in v)
    if t=='point2': return isinstance(v,list) and len(v)==2 and all(is_number(x) for x in v)
    return False

def close_value(got, exp, tol):
    if isinstance(exp,bool) or isinstance(exp,str) or exp is None: return got==exp
    if isinstance(exp,(int,float)) and not isinstance(exp,bool):
        return is_number(got) and math.isclose(float(got),float(exp),rel_tol=tol,abs_tol=tol)
    if isinstance(exp,list): return isinstance(got,list) and len(got)==len(exp) and all(close_value(g,e,tol) for g,e in zip(got,exp))
    if isinstance(exp,dict): return isinstance(got,dict) and got.keys()==exp.keys() and all(close_value(got[k],v,tol) for k,v in exp.items())
    return got==exp

def run_golden_oracles(c):
    cases=json.loads((ROOT/'golden/cases.json').read_text(encoding='utf-8-sig'))['cases']
    rows=[]
    for case in cases:
        try:
            sm=c.tool_call('yk.compute',{'op':case['op'],'a':case['a']}); p=parse_payload(sm)
            if 'error' in case:
                ok=p.get('e')==case['error']; got=p.get('e')
            else:
                got=p.get('r'); ok=close_value(got,case['expect'],float(case.get('tol',1e-10)))
        except Exception as e:
            ok=False; got='RPC:'+str(e)
        rows.append({'id':case['id'],'category':case['category'],'op':case['op'],'ok':ok,'got':got})
    return rows

def special_control_call(c,op):
    if op=='udo.formula': a=[{'op':'user.audit_formula','p':['x'],'expr':'x+1'}]
    elif op=='udo.composite': a=[{'op':'user.audit_composite','p':['x'],'pipe':[{'op':'math.add','a':['$a0',1]}]}]
    elif op=='udo.remove': a=['user.audit_missing']
    elif op=='udo.list': a=[]
    elif op=='udo.export':
        # ensure dependency exists
        c.tool_call('yk.compute',{'op':'udo.formula','a':[{'op':'user.audit_formula','p':['x'],'expr':'x+1'}]})
        a=[{'name':'audit','ops':['user.audit_formula']}]
    elif op=='udo.import': a=[{'v':1,'name':'auditpack','ops':[{'k':'f','op':'pack.auditpack.inc','p':['x'],'expr':'x+1'}]}]
    elif op=='udo.uninstall':
        c.tool_call('yk.compute',{'op':'udo.import','a':[{'v':1,'name':'auditpack','ops':[{'k':'f','op':'pack.auditpack.inc','p':['x'],'expr':'x+1'}]}]})
        a=['auditpack']
    else:return None,None
    sm=c.tool_call('yk.compute',{'op':op,'a':a});return a,sm

def percentile(xs,p):
    if not xs:return 0.0
    ys=sorted(xs);q=(len(ys)-1)*p;lo=math.floor(q);hi=math.ceil(q)
    return ys[lo] if lo==hi else ys[lo]*(hi-q)+ys[hi]*(q-lo)

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--exe',required=True);ap.add_argument('--out',default='full_audit_results/latest');ap.add_argument('--strict',action='store_true');ap.add_argument('--max-candidates',type=int,default=192);args=ap.parse_args()
    ops=registry_ops(); print(f'opcode enumeration {len(ops)}/{EXPECTED_OPS}')
    primary=load_primary_fixtures(); out=Path(args.out);out.mkdir(parents=True,exist_ok=True)
    enc=tiktoken.get_encoding('o200k_base') if tiktoken else None
    learned={};discovery=[]
    with tempfile.TemporaryDirectory(prefix='yk-full-audit-discover-') as home:
        with StdioMcpClient(str(Path(args.exe).resolve()),timeout=30,env={'YEKATERINA_HOME':home}) as c:
            tools=[x.get('name') for x in c.tools_list().response.get('result',{}).get('tools',[])]
            spec_valid=0
            specs=[]
            for op in ops:
                try:
                    ss=c.tool_call('yk.spec',{'op':op}); sp=json.loads(mcp_text(ss.response)); sig=sp.get('a',[]) if isinstance(sp,dict) else []; ret=sp.get('r') if isinstance(sp,dict) else None
                    if sp.get('op')==op and isinstance(sig,list) and isinstance(ret,str): spec_valid+=1
                    else: sig=[]; ret=None
                except Exception: sig=[]; ret=None
                specs.append((op,sig,ret))
            print(f'spec coverage {spec_valid}/{len(ops)}')
            if spec_valid != len(ops):
                bad=[op for op,sig,ret in specs if not isinstance(ret,str)]
                raise RuntimeError(f'live yk.spec coverage incomplete: {spec_valid}/{len(ops)}; examples={bad[:12]}')
            for idx,(op,sig,ret) in enumerate(specs,1):
                fam=op.split('.',1)[0]; attempts=[]; success=False; chosen=None; last=None
                if op.startswith('udo.'):
                    try:
                        chosen,sm=special_control_call(c,op);p=parse_payload(sm);attempts.append({'a':chosen,'e':p.get('e')});success='r' in p and type_ok(p.get('r'),ret);last=p
                    except Exception as e:last={'e':'RPC','detail':str(e)}
                else:
                    for n,a in enumerate(candidates(op,sig,primary)):
                        if n>=args.max_candidates:break
                        try:
                            sm=c.tool_call('yk.compute',{'op':op,'a':a});p=parse_payload(sm);attempts.append({'a':a,'e':p.get('e')});last=p
                            if 'r' in p and type_ok(p.get('r'),ret): success=True;chosen=a;break
                        except Exception as e:last={'e':'RPC','detail':str(e)};attempts.append({'a':a,'e':'RPC'})
                if success:learned[op]=chosen
                discovery.append({'op':op,'family':fam,'success':success,'fixture':chosen,'last':last,'attempts':len(attempts),'errors':[x['e'] for x in attempts if x['e']]})
                if idx%100==0:print(f'fixture discovery {idx}/{len(specs)} success={sum(x["success"] for x in discovery)}')
    # replay learned fixtures for clean one-call-per-op metrics
    rows=[]
    with tempfile.TemporaryDirectory(prefix='yk-full-audit-replay-') as home:
        with StdioMcpClient(str(Path(args.exe).resolve()),timeout=30,env={'YEKATERINA_HOME':home}) as c:
            for op,_sig,ret in specs:
                if op not in learned:continue
                try:
                    if op.startswith('udo.'):
                        a,sm=special_control_call(c,op)
                    else:
                        a=learned[op];sm=c.tool_call('yk.compute',{'op':op,'a':a})
                    p=parse_payload(sm);ok='r' in p and type_ok(p.get('r'),ret)
                    rows.append({'op':op,'family':op.split('.',1)[0],'ok':ok,'return_type':ret,'ms':sm.elapsed_ms,'wire_tokens':wire_tokens(sm,enc),'error':p.get('e')})
                except Exception as e:rows.append({'op':op,'family':op.split('.',1)[0],'ok':False,'ms':0.0,'wire_tokens':None,'error':'RPC:'+str(e)})
    total=len(specs);found=len(learned);replay_ok=sum(r['ok'] for r in rows);missing=[x for x in discovery if not x['success']]
    if missing:
        print('missing fixtures: '+', '.join(x['op'] for x in missing))
    with tempfile.TemporaryDirectory(prefix='yk-full-audit-oracle-') as home:
        with StdioMcpClient(str(Path(args.exe).resolve()),timeout=30,env={'YEKATERINA_HOME':home}) as c:
            oracle_rows=run_golden_oracles(c)
    oracle_pass=sum(r['ok'] for r in oracle_rows); oracle_total=len(oracle_rows)
    fams={}
    for fam in sorted(set(op.split('.',1)[0] for op,_,_ in specs)):
        expected=sum(1 for op,_,_ in specs if op.startswith(fam+'.'))
        rr=[r for r in rows if r['family']==fam]; dd=[d for d in discovery if d['family']==fam]
        lats=[r['ms'] for r in rr if r['ok']]; toks=[r['wire_tokens'] for r in rr if r['ok'] and r['wire_tokens'] is not None]
        fams[fam]={'registered':expected,'fixture_success':sum(d['success'] for d in dd),'replay_success':sum(r['ok'] for r in rr),'p50_ms':percentile(lats,.5),'p95_ms':percentile(lats,.95),'wire_tokens':sum(toks) if toks else None}
    result={'benchmark':'yekaterina-full-capability-audit-alpha12','expected_registered':EXPECTED_OPS,'enumeration_pass':total==EXPECTED_OPS,'scope':'all registered opcodes: live spec + executable fixture + return-type contract; golden oracle is reported separately','registered':total,'spec_valid':spec_valid,'spec_coverage':spec_valid/total if total else 1.0,'fixture_discovered':found,'fixture_coverage':found/total if total else 1.0,'replay_success':replay_ok,'replay_coverage':replay_ok/total if total else 1.0,'golden_oracle_passed':oracle_pass,'golden_oracle_total':oracle_total,'golden_oracle_accuracy':oracle_pass/oracle_total if oracle_total else 1.0,'mcp_tools':tools,'tool_surface_pass':sorted(tools)==['yk.compute','yk.find','yk.spec'],'families':fams,'missing':[{'op':x['op'],'errors':x['errors'],'attempts':x['attempts'],'last':x.get('last')} for x in missing],'oracle_failures':[r for r in oracle_rows if not r['ok']]}
    (out/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    (out/'learned_fixtures.json').write_text(json.dumps(learned,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    lines=['# Yekaterina v1.0.0 Full Capability Audit','',f'- Registered opcodes: **{total}**',f'- Valid fixtures discovered: **{found}/{total} ({result["fixture_coverage"]*100:.2f}%)**',f'- Clean replay success: **{replay_ok}/{total} ({result["replay_coverage"]*100:.2f}%)**',f'- MCP tool surface: **{"PASS" if result["tool_surface_pass"] else "FAIL"}**',f'- `yk.spec` coverage: **{spec_valid}/{total}**',f'- Golden oracle correctness: **{oracle_pass}/{oracle_total} ({result["golden_oracle_accuracy"]*100:.2f}%)**','', '> `1215/1215` below means live execution/type coverage, not proof that every mathematical result is correct.','', '| Family | Registered | Fixtures | Replay | p50 | p95 | Wire tokens |','|---|---:|---:|---:|---:|---:|---:|']
    for fam,d in fams.items():lines.append(f'| {fam} | {d["registered"]} | {d["fixture_success"]} | {d["replay_success"]} | {d["p50_ms"]:.3f} ms | {d["p95_ms"]:.3f} ms | {d["wire_tokens"] if d["wire_tokens"] is not None else "n/a"} |')
    if missing:
        lines+=['','## Missing valid fixtures / unexpected failures']
        for x in missing:lines.append(f'- `{x["op"]}` — attempts `{x["attempts"]}`, errors `{x["errors"][-8:]}`')
    else:lines+=['','**FULL OPCODE EXECUTION/TYPE COVERAGE GATE: PASS**']
    if oracle_pass!=oracle_total:
        lines+=['','## Golden oracle failures']
        for r in oracle_rows:
            if not r['ok']: lines.append(f'- `{r["id"]}` / `{r["op"]}` got `{r["got"]}`')
    (out/'REPORT.md').write_text('\n'.join(lines)+'\n',encoding='utf-8')
    print(f'Full audit fixtures: {found}/{total} ({result["fixture_coverage"]*100:.2f}%)')
    print(f'Clean replay/type contract: {replay_ok}/{total} ({result["replay_coverage"]*100:.2f}%)')
    print(f'Report: {out/"REPORT.md"}')
    ok=(total==EXPECTED_OPS and result['tool_surface_pass'] and spec_valid==total and found==total and replay_ok==total and oracle_pass==oracle_total)
    if args.strict and not ok:raise SystemExit(1)

if __name__=='__main__':main()
