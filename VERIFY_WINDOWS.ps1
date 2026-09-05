$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "============================================================"
Write-Host " Yekaterina v1.0.0 verification"
Write-Host "============================================================"

python .\scripts\static_audit.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

python .\scripts\lexical_rust_audit.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

python .\scripts\operation_manifest.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

python .\scripts\validate_golden_manifest.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

python .\scripts\validate_full_audit.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

if (-not (Test-Path .\Cargo.lock)) {
    Write-Host "Cargo.lock missing: generating the first reproducibility lockfile..."
    cargo generate-lockfile
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

cargo test --locked --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo clippy --locked --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo build --locked --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "PASS: v1.0.0 local verification completed"
Write-Host "Binary: .\target\release\yekaterina.exe"
Write-Host "Running v1.0.0 MCP golden correctness suite..."
python .\golden\run_golden.py --exe ".\target\release\yekaterina.exe" --out ".\golden_results\latest"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "PASS: v1.0.0 local + MCP golden verification completed"
Write-Host "Golden report: .\golden_results\latest\REPORT.md"
Write-Host "Next: run RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat, then run the Self-Regression Benchmark."
