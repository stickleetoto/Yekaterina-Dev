@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0VERIFY_V11_WINDOWS.ps1"
exit /b %ERRORLEVEL%
