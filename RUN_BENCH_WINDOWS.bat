@echo off
setlocal
cd /d "%~dp0"
echo ============================================================
echo  Yekaterina MCP Benchmark
echo ============================================================
if not exist ".\target\release\yekaterina.exe" (
  echo FAIL: release binary not found. Run VERIFY_V11_WINDOWS.bat first.
  exit /b 1
)
set BASELINE=.\bench_results\v1.0.0-frozen\result.json
if exist "%BASELINE%" (
  python .\bench\run_bench.py --exe ".\target\release\yekaterina.exe" --out ".\bench_results\latest" --compare "%BASELINE%"
) else (
  echo NOTE: no frozen v1.0.0 baseline found; running without comparison.
  python .\bench\run_bench.py --exe ".\target\release\yekaterina.exe" --out ".\bench_results\latest"
)
exit /b %ERRORLEVEL%
