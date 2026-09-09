#!/usr/bin/env python3
"""Yekaterina v1.3 Compute Program alpha.1 MCP smoke/demo.

Uses only the Python standard library and talks to the release executable over
real MCP stdio. The central assertion is that a named, forward-referencing
calculation graph executes through one yk.compute tool call.
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


def recv(messages: "queue.Queue[dict[str, Any]]", want_id: int, timeout: float = 10.0) -> dict[str, Any]:
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
        raise DemoError(f"tool response contained no text: {response!r}")
    try:
        return json.loads(" ".join(texts))
    except json.JSONDecodeError as exc:
        raise DemoError(f"tool returned non-JSON text: {texts!r}") from exc


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: demo_v13.py <path-to-yekaterina-executable>", file=sys.stderr)
        return 2

    proc = subprocess.Popen(
        [sys.argv[1]],
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
        send(proc, {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params})
        return recv(messages, request_id)

    try:
        init = request(
            "initialize",
            {
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "yekaterina-v13-demo", "version": "1.0"},
            },
        )
        if "error" in init:
            raise DemoError(f"initialize failed: {init['error']!r}")
        send(proc, {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}})

        listed = request("tools/list", {})
        if "error" in listed:
            raise DemoError(f"tools/list failed: {listed['error']!r}")
        tools = listed.get("result", {}).get("tools", [])
        names = [tool.get("name") for tool in tools if isinstance(tool, dict)]
        if sorted(names) != sorted(EXPECTED_TOOLS):
            raise DemoError(f"tool surface changed: {names!r}")

        compute_schema = next(
            (tool.get("inputSchema", {}) for tool in tools if tool.get("name") == "yk.compute"),
            {},
        )
        properties = compute_schema.get("properties", {}) if isinstance(compute_schema, dict) else {}
        if "program" not in properties:
            raise DemoError("yk.compute schema does not advertise the v1.3 program field")

        # One and only one yk.compute call for the whole calculation program.
        # `scaled` deliberately references `sum` before sum is declared, proving
        # that declaration order is not execution order.
        program = content_json(
            request(
                "tools/call",
                {
                    "name": "yk.compute",
                    "arguments": {
                        "input": {"x": [1, 2, 3, 4]},
                        "program": {
                            "steps": [
                                {"id": "scaled", "op": "math.mul", "a": ["$sum", 10]},
                                {"id": "sum", "op": "stat.sum", "a": ["$input.x"]},
                            ],
                            "return": "scaled",
                        },
                    },
                },
            )
        )
        if not isinstance(program, dict) or program.get("r") != 100:
            raise DemoError(f"compute program returned the wrong result: {program!r}")

        print("Yekaterina v1.3 Compute Program alpha.1")
        print("MCP tools:", ", ".join(names))
        print("program: sum([1,2,3,4]) -> *10 ->", program["r"])
        print("yk.compute calls for calculation graph: 1")
        print("V13 PROGRAM DEMO PASS")
        return 0
    except Exception as exc:
        print(f"V13 PROGRAM DEMO FAIL: {exc}", file=sys.stderr)
        if stderr_tail:
            print("server stderr tail:", file=sys.stderr)
            for line in stderr_tail:
                print(f"  {line}", file=sys.stderr)
        return 1
    finally:
        proc.kill()


if __name__ == "__main__":
    raise SystemExit(main())
