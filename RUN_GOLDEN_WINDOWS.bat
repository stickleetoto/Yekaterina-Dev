@echo off
setlocal
cd /d "%~dp0"
echo ============================================================
echo  Yekaterina v1.0.0 MCP Golden Validation
echo ============================================================
if not exist ".\target\release\yekaterina.exe" (
  echo Release binary not found. Building...
  cargo build --release || exit /b 1
)
python .\golden\run_golden.py --exe ".\target\release\yekaterina.exe" --out ".\golden_results\latest"
exit /b %ERRORLEVEL%
