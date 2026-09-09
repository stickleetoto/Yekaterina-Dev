@echo off
setlocal

set "EXE="
if not "%~1"=="" set "EXE=%~f1"

pushd "%~dp0" >nul

if not defined EXE set "EXE=%CD%\target\release\yekaterina.exe"

if not exist "%EXE%" (
  echo [Yekaterina demo] Release binary not found. Building with the locked dependency set...
  cargo build --locked --release
  if errorlevel 1 (
    set "RC=%ERRORLEVEL%"
    popd >nul
    exit /b %RC%
  )
)

where python >nul 2>&1
if errorlevel 1 (
  echo [Yekaterina demo] Python 3 is required to run tools\demo.py.
  popd >nul
  exit /b 2
)

python "%CD%\tools\demo.py" "%EXE%"
set "RC=%ERRORLEVEL%"

popd >nul
exit /b %RC%
