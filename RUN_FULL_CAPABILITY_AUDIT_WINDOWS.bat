@echo off
setlocal
cd /d "%~dp0"
echo ============================================================
echo  Yekaterina v1.0.0 Full Capability Audit
 echo ============================================================
if not exist ".\target\release\yekaterina.exe" (
  echo FAIL: release binary not found. Run VERIFY_WINDOWS.bat first.
  exit /b 1
)
python -m pip install "tiktoken>=0.7" >nul
if errorlevel 1 exit /b %errorlevel%
python .\scripts\validate_full_audit.py
if errorlevel 1 exit /b %errorlevel%
python .\full_audit\run_full_audit.py --exe ".\target\release\yekaterina.exe" --out ".\full_audit_results\latest" --strict
if errorlevel 1 (
  echo.
  echo FULL AUDIT: FAIL - inspect .\full_audit_results\latest\REPORT.md
  exit /b 1
)
echo.
echo PASS: all 1215 opcodes executed through MCP and matched their return-type contracts.
echo PASS: golden oracle correctness also passed at 100%%.
echo Report: .\full_audit_results\latest\REPORT.md
