from __future__ import annotations
import argparse, json, math, statistics, tempfile, time
from pathlib import Path
from typing import Any
from mcp_client import StdioMcpClient, mcp_text

def pct(xs,q):
    if not xs:return 0.0
    ys=sorted(xs); p=(len(ys)-1)*q; lo=math.floor(p); hi=math.ceil(p)
    return ys[lo] if lo==hi else ys[lo]*(hi-p)+ys[hi]*(p-lo)

def close_value(got:Any, exp:Any, tol:float)->bool:
    if isinstance(exp,bool) or isinstance(exp,str) or exp is None: return got==exp
    if isinstance(exp,(int,float)) and not isinstance(exp,bool):
        if not isinstance(got,(int,float)) or isinstance(got,bool): return False
        return math.isclose(float(got),float(exp),rel_tol=tol,abs_tol=tol)
    if isinstance(exp,list):
        return isinstance(got,list) and len(got)==len(exp) and all(close_value(g,e,tol) for g,e in zip(got,exp))
    if isinstance(exp,dict):
        return isinstance(got,dict) and got.keys()==exp.keys() and all(close_value(got[k],v,tol) for k,v in exp.items())
    return got==exp

def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--exe',required=True)
    ap.add_argument('--cases',default=str(Path(__file__).with_name('cases.json')))
    ap.add_argument('--out',default='golden_results/latest')
    args=ap.parse_args()
    cases=json.loads(Path(args.cases).read_text(encoding='utf-8-sig'))['cases']
    out=Path(args.out); out.mkdir(parents=True,exist_ok=True)
    rows=[]; tools=[]
    with tempfile.TemporaryDirectory(prefix='yk-golden-') as home:
        with StdioMcpClient(str(Path(args.exe).resolve()),env={'YEKATERINA_HOME':home}) as c:
            tls=c.tools_list().response.get('result',{}).get('tools',[])
            tools=[x.get('name') for x in tls]
            for case in cases:
                t0=time.perf_counter_ns()
                sample=c.tool_call('yk.compute',{'op':case['op'],'a':case['a']})
                elapsed=(time.perf_counter_ns()-t0)/1_000_000
                text=mcp_text(sample.response)
                try: payload=json.loads(text)
                except Exception: payload={'__parse_error__':text}
                if 'error' in case:
                    passed=payload.get('e')==case['error']; got=payload.get('e')
                else:
                    got=payload.get('r'); passed=close_value(got,case['expect'],float(case.get('tol',1e-10)))
                rows.append({**case,'got':got,'pass':passed,'ms':elapsed})
    cats={}
    for r in rows:
        d=cats.setdefault(r['category'],{'total':0,'passed':0,'latencies':[],'failed':[]})
        d['total']+=1; d['passed']+=int(r['pass']); d['latencies'].append(r['ms'])
        if not r['pass']: d['failed'].append({'id':r['id'],'op':r['op'],'expect':r.get('expect',{'error':r.get('error')}),'got':r['got']})
    for d in cats.values():
        d['accuracy']=d['passed']/d['total'] if d['total'] else 1.0
        d['p50_ms']=pct(d.pop('latencies'),.50); d['p95_ms']=pct([r['ms'] for r in rows if r['category'] in []],.95) if False else 0.0
    # compute p95 separately without keeping giant lists in output
    for name,d in cats.items(): d['p95_ms']=pct([r['ms'] for r in rows if r['category']==name],.95)
    passed=sum(int(r['pass']) for r in rows); total=len(rows)
    tool_ok=sorted(tools)==['yk.compute','yk.find','yk.spec']
    result={'benchmark':'yekaterina-golden-alpha12','scope':'MCP-only golden correctness','tools':tools,'tool_surface_pass':tool_ok,
            'total':total,'passed':passed,'accuracy':passed/total if total else 1.0,'categories':cats,
            'failures':[{'category':r['category'],'id':r['id'],'op':r['op'],'expect':r.get('expect',{'error':r.get('error')}),'got':r['got']} for r in rows if not r['pass']]}
    (out/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
    lines=['# Yekaterina v1.0.0 Golden MCP Validation','',f'- MCP tool surface: **{"PASS" if tool_ok else "FAIL"}** (`{len(tools)}` tools)',f'- Cases: **{passed}/{total}**',f'- Overall accuracy: **{result["accuracy"]*100:.2f}%**','', '| Category | Passed | Accuracy | p50 | p95 |','|---|---:|---:|---:|---:|']
    for name,d in sorted(cats.items()): lines.append(f'| {name} | {d["passed"]}/{d["total"]} | {d["accuracy"]*100:.2f}% | {d["p50_ms"]:.3f} ms | {d["p95_ms"]:.3f} ms |')
    if result['failures']:
        lines += ['','## Failures']
        for f in result['failures']: lines.append(f'- `{f["id"]}` / `{f["op"]}`: expected `{f["expect"]}`, got `{f["got"]}`')
    else: lines += ['','**GOLDEN GATE: PASS**']
    (out/'REPORT.md').write_text('\n'.join(lines)+'\n',encoding='utf-8')
    print(f'Golden cases: {passed}/{total} ({result["accuracy"]*100:.2f}%)')
    print(f'MCP tools: {len(tools)} -> {tools}')
    print(f'Report: {out / "REPORT.md"}')
    raise SystemExit(0 if passed==total and tool_ok else 1)

if __name__=='__main__': main()
