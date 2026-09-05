from __future__ import annotations
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
files=list((ROOT/'src').glob('*.rs'))+list((ROOT/'tests').glob('*.rs'))
PAIRS={')':'(',']':'[','}':'{'}
for p in files:
    s=p.read_text(encoding='utf-8')
    stack=[]; i=0; state='code'; raw_hashes=0
    while i<len(s):
        c=s[i]; n=s[i+1] if i+1<len(s) else ''
        if state=='code':
            if c=='/' and n=='/': state='line'; i+=2; continue
            if c=='/' and n=='*': state='block'; depth=1; i+=2; continue
            if c=='"': state='str'; i+=1; continue
            if c=="'":
                # Treat only obvious character literals as chars; lifetimes remain code.
                if i+2<len(s) and (s[i+2]=="'" or (s[i+1]=='\\' and i+3<len(s) and s[i+3]=="'")):
                    state='char'; i+=1; continue
            if c in '([{': stack.append((c,i))
            elif c in ')]}':
                if not stack or stack[-1][0]!=PAIRS[c]: raise SystemExit(f'FAIL: delimiter mismatch {p}:{i} {c}')
                stack.pop()
            i+=1; continue
        if state=='line':
            if c=='\n': state='code'
            i+=1; continue
        if state=='block':
            if c=='/' and n=='*': depth+=1; i+=2; continue
            if c=='*' and n=='/': depth-=1; i+=2; state='code' if depth==0 else 'block'; continue
            i+=1; continue
        if state in {'str','char'}:
            if c=='\\': i+=2; continue
            if (state=='str' and c=='"') or (state=='char' and c=="'"): state='code'
            i+=1; continue
    if state in {'str','char','block'}: raise SystemExit(f'FAIL: unterminated {state} in {p}')
    if stack: raise SystemExit(f'FAIL: unclosed delimiter in {p}: {stack[-1]}')
print(f'PASS: lexical Rust delimiter audit {len(files)} files')
