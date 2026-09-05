"""Benchmark-side MCP stdio client.

Deliberately a thin subclass of the frozen ``golden/mcp_client.py`` rather than a
copy or a modification of it.  The golden client is part of the v1.0.0
correctness gate and must not change; the benchmark needs two extra
capabilities the golden client does not have:

1. argv passthrough, so ``--workers N`` can be given to the server binary
   (v1.0.0 ignores unknown argv, which is what makes the same harness usable
   against both the frozen baseline and v1.1),
2. access to the child pid, so RSS and CPU time can be sampled.
"""
from __future__ import annotations

import os
import subprocess
import threading
from pathlib import Path
from typing import Any

import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "golden"))
from mcp_client import StdioMcpClient, mcp_text  # noqa: E402,F401  (re-exported)

try:
    import psutil  # type: ignore
except Exception:  # pragma: no cover - psutil is optional
    psutil = None


class BenchClient(StdioMcpClient):
    """StdioMcpClient plus argv and process-metric support."""

    def __init__(self, command: str, argv: list[str] | None = None, **kwargs: Any):
        super().__init__(command, **kwargs)
        self.argv = list(argv or [])

    def start(self) -> None:
        env = os.environ.copy()
        env.update(self.env)
        self.proc = subprocess.Popen(
            [self.command, *self.argv],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
            env=env,
        )
        threading.Thread(target=self._pump_stdout, daemon=True).start()
        threading.Thread(target=self._pump_stderr, daemon=True).start()
        self._request(
            "initialize",
            {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "yekaterina-bench", "version": "1.1"},
            },
        )
        self._send({"jsonrpc": "2.0", "method": "notifications/initialized"})

    # ---- process metrics -------------------------------------------------

    def metrics(self) -> dict[str, Any]:
        """Peak RSS (bytes) and CPU time (seconds) for the server process.

        Returns ``{}`` when psutil is unavailable so the harness degrades
        gracefully in CI rather than failing.
        """
        if psutil is None or self.proc is None:
            return {}
        try:
            p = psutil.Process(self.proc.pid)
            with p.oneshot():
                mem = p.memory_info()
                cpu = p.cpu_times()
            out = {
                "rss_bytes": int(mem.rss),
                "cpu_user_s": float(cpu.user),
                "cpu_system_s": float(cpu.system),
                "num_threads": int(p.num_threads()),
            }
            peak = getattr(mem, "peak_wset", None)
            if peak is not None:
                out["peak_rss_bytes"] = int(peak)
            return out
        except Exception:
            return {}
