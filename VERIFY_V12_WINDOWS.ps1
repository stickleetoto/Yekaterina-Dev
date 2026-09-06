$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "============================================================"
Write-Host " Yekaterina v1.2 verification"
Write-Host "============================================================"
Write-Host ""
Write-Host "scripts/static_audit.py (v1.0.0) and scripts/static_audit_v11.py (v1.1)"
Write-Host "are preserved unmodified as frozen historical artifacts. Neither can"
Write-Host "pass on this tree: each gates the operation count of its own release"
Write-Host "line, and v1.2 raised it from 1215 to 1284. The v1.2 audit below"
Write-Host "reproduces every gate from both, and verifies by hash that neither"
Write-Host "earlier audit has been tampered with."
Write-Host ""

python .\scripts\static_audit_v12.py
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
Write-Host "PASS: v1.2 local verification completed"
Write-Host "Binary: .\target\release\yekaterina.exe"
Write-Host ""
Write-Host "Verifying the v1.2 operations against independently computed values..."
python .\scriptserify_v12_operations.py
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host ""
Write-Host "Running the frozen v1.0.0 MCP golden correctness suite (expectations unchanged)..."
python .\golden\run_golden.py --exe ".\target\release\yekaterina.exe" --out ".\golden_results\latest"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "PASS: v1.2 local + MCP golden verification completed"
Write-Host "Golden report: .\golden_results\latest\REPORT.md"
Write-Host ""
Write-Host "Next:"
Write-Host "  1. .\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat"
Write-Host "  2. .\RUN_BENCH_WINDOWS.bat   (compares against the frozen v1.0.0 baseline)"
