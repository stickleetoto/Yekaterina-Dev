#!/usr/bin/env python3
"""Reviewer-facing Yekaterina MCP demo.

Only the Python standard library is used. The executable is treated exactly as
an MCP host treats it: a subprocess speaking newline-delimited JSON-RPC over
stdio.
"""
from __future__ import annotations

import json
import queue
import subprocess
import sys
import threading
import time
from collections import deque
from typing import Any

PROTOCOL_VERSION = "2025-06-18"
EXPECTED_TOOLS = ["yk.compute", "yk.find", "yk.spec"]


class DemoError(RuntimeError):
    pass


def send(proc: subprocess.Popen[str], obj: dict[str, Any]) -> None:
    assert proc.stdin is not None
    proc.stdin.write(json.dumps(obj, separators=(",", ":")) + "\n")
    proc.stdin.flush()


def recv(
    messages: "queue.Queue[dict[str, Any]]",
    want_id: int,
    timeout: float = 10.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout
    while time.time() < deadline:
        try:
            obj = messages.get(timeout=max(0.05, deadline - time.time()))
        except queue.Empty:
            continue
        if obj.get("id") == want_id:
            return obj
    raise DemoError(f"timeout waiting for JSON-RPC id={want_id}")


def content_json(response: dict[str, Any]) -> Any:
    if "error" in response:
        raise DemoError(f"JSON-RPC error: {response['error']!r}")
    content = response.get("result", {}).get("content", [])
    texts = [
        item.get("text", "")
        for item in content
        if isinstance(item, dict) and isinstance(item.get("text"), str)
    ]
    if not texts:
        raise DemoError(f"tool response contained no text content: {response!r}")
    text = " ".join(texts)
    try:
        return json.loads(text)
    except json.JSONDecodeError as exc:
        raise DemoError(f"tool returned non-JSON text: {text!r}") from exc


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: demo.py <path-to-yekaterina-executable>", file=sys.stderr)
        return 2

    exe = sys.argv[1]
    proc = subprocess.Popen(
        [exe],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )

    messages: "queue.Queue[dict[str, Any]]" = queue.Queue()
    stderr_tail: deque[str] = deque(maxlen=20)

    def read_stdout() -> None:
        assert proc.stdout is not None
        for line in proc.stdout:
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(obj, dict):
                messages.put(obj)

    def read_stderr() -> None:
        assert proc.stderr is not None
        for line in proc.stderr:
            stderr_tail.append(line.rstrip())

    threading.Thread(target=read_stdout, daemon=True).start()
    threading.Thread(target=read_stderr, daemon=True).start()

    next_id = 1

    def request(method: str, params: dict[str, Any]) -> dict[str, Any]:
        nonlocal next_id
        request_id = next_id
        next_id += 1
        send(
            proc,
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": params,
            },
        )
        return recv(messages, request_id)

    def call_tool(name: str, arguments: dict[str, Any]) -> Any:
        return content_json(
            request(
                "tools/call",
                {"name": name, "arguments": arguments},
            )
        )

    try:
        init = request(
            "initialize",
            {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "yekaterina-rc-demo",
                    "version": "1.0",
                },
            },
        )
        if "error" in init:
            raise DemoError(f"initialize failed: {init['error']!r}")

        send(
            proc,
            {
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {},
            },
        )

        tools_response = request("tools/list", {})
        if "error" in tools_response:
            raise DemoError(f"tools/list failed: {tools_response['error']!r}")
        names = [
            tool.get("name")
            for tool in tools_response.get("result", {}).get("tools", [])
            if isinstance(tool, dict)
        ]
        if sorted(names) != sorted(EXPECTED_TOOLS):
            raise DemoError(f"unexpected MCP tool surface: {names!r}")

        add = call_tool("yk.compute", {"op": "math.add", "a": [20, 22]})
        if not isinstance(add, dict) or add.get("r") != 42:
            raise DemoError(f"math.add changed: {add!r}")

        decimal = call_tool(
            "yk.compute",
            {"op": "dec.add", "a": ["0.1", "0.2"]},
        )
        if not isinstance(decimal, dict) or str(decimal.get("r")) != "0.3":
            raise DemoError(f"dec.add exactness changed: {decimal!r}")

        batch = call_tool(
            "yk.compute",
            {
                "ops": [
                    ["math.add", 1, 2],
                    ["math.mul", 6, 7],
                    ["stat.sum", [1, 2, 3, 4]],
                ]
            },
        )
        expected_batch = [3, 42, 10]
        if not isinstance(batch, dict) or batch.get("r") != expected_batch:
            raise DemoError(f"batch result changed: {batch!r}")

        found = call_tool("yk.find", {"q": "welch", "l": 5})
        hits = found.get("r") if isinstance(found, dict) else None
        if not isinstance(hits, list):
            raise DemoError(f"yk.find returned an unexpected shape: {found!r}")
        welch = next(
            (
                hit
                for hit in hits
                if isinstance(hit, str) and "welch" in hit.lower()
            ),
            None,
        )
        if welch is None:
            raise DemoError(f"yk.find could not discover a Welch operation: {hits!r}")

        spec = call_tool("yk.spec", {"op": welch})
        if not isinstance(spec, dict) or spec.get("op") != welch:
            raise DemoError(f"yk.spec returned an unexpected contract: {spec!r}")

        print("Yekaterina v1.2 RC demo")
        print("MCP tools:", ", ".join(names))
        print("math.add: 20 + 22 ->", add["r"])
        print("dec.add: 0.1 + 0.2 ->", decimal["r"])
        print("batch:", batch["r"])
        print('discovery "welch" ->', welch)
        print(
            f"spec {welch}: args={spec.get('a')!r} returns={spec.get('r')!r}"
        )
        print("DEMO PASS")
        return 0

    except Exception as exc:
        print(f"DEMO FAIL: {exc}", file=sys.stderr)
        if stderr_tail:
            print("server stderr tail:", file=sys.stderr)
            for line in stderr_tail:
                print(f"  {line}", file=sys.stderr)
        return 1
    finally:
        proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
