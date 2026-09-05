from __future__ import annotations
import json, os, queue, subprocess, threading, time
from dataclasses import dataclass
from typing import Any

class McpError(RuntimeError):
    pass

@dataclass
class RpcSample:
    request: dict[str, Any]
    response: dict[str, Any]
    elapsed_ms: float

class StdioMcpClient:
    def __init__(self, command: str, timeout: float = 15.0, env: dict[str,str] | None = None):
        self.command=command; self.timeout=timeout; self.env=env or {}; self.proc=None
        self._stdout_q: queue.Queue[str]=queue.Queue(); self._stderr=[]; self._id=0
    def __enter__(self): self.start(); return self
    def __exit__(self, exc_type, exc, tb): self.close()
    def start(self):
        env=os.environ.copy(); env.update(self.env)
        self.proc=subprocess.Popen([self.command],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,
            text=True,encoding='utf-8',errors='replace',bufsize=1,env=env)
        threading.Thread(target=self._pump_stdout,daemon=True).start(); threading.Thread(target=self._pump_stderr,daemon=True).start()
        self._request('initialize',{'protocolVersion':'2024-11-05','capabilities':{},'clientInfo':{'name':'yekaterina-golden','version':'0.1'}})
        self._send({'jsonrpc':'2.0','method':'notifications/initialized'})
    def _pump_stdout(self):
        for line in self.proc.stdout:
            line=line.strip()
            if line:self._stdout_q.put(line)
    def _pump_stderr(self):
        for line in self.proc.stderr:
            if len(self._stderr)<200:self._stderr.append(line.rstrip())
    def _send(self,obj):
        self.proc.stdin.write(json.dumps(obj,separators=(',',':'),ensure_ascii=False)+'\n'); self.proc.stdin.flush()
    def _request(self,method,params=None):
        self._id+=1; rid=self._id; req={'jsonrpc':'2.0','id':rid,'method':method}
        if params is not None:req['params']=params
        t0=time.perf_counter_ns(); self._send(req); deadline=time.monotonic()+self.timeout
        while time.monotonic()<deadline:
            try: line=self._stdout_q.get(timeout=max(.01,deadline-time.monotonic()))
            except queue.Empty: break
            try: msg=json.loads(line)
            except json.JSONDecodeError: continue
            if msg.get('id')!=rid: continue
            if 'error' in msg: raise McpError(f"{method}: {msg['error']}")
            return RpcSample(req,msg,(time.perf_counter_ns()-t0)/1_000_000)
        raise McpError('timeout: '+method+'\n'+'\n'.join(self._stderr[-12:]))
    def tools_list(self): return self._request('tools/list',{})
    def tool_call(self,name,arguments): return self._request('tools/call',{'name':name,'arguments':arguments})
    def close(self):
        if not self.proc:return
        try:self.proc.stdin.close()
        except Exception:pass
        try:self.proc.terminate(); self.proc.wait(timeout=2)
        except Exception:
            try:self.proc.kill()
            except Exception:pass
        self.proc=None

def mcp_text(resp: dict[str,Any])->str:
    result=resp.get('result',{}); content=result.get('content',[]) if isinstance(result,dict) else []
    return '\n'.join(str(x.get('text','')) for x in content if isinstance(x,dict) and x.get('type')=='text')
