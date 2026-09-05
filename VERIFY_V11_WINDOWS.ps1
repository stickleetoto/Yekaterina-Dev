$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

Write-Host "============================================================"
Write-Host " Yekaterina v1.1 verification"
Write-Host "============================================================"
Write-Host ""
Write-Host "The v1.0.0 audit (scripts/static_audit.py) is preserved unmodified as a"
Write-Host "frozen historical artifact. The v1.1 audit below reproduces every v1.0.0"
Write-Host "gate and adds the concurrency gates; it also verifies the v1.0.0 audit"
Write-Host "has not been tampered with."
Write-Host ""

python .\scripts\static_audit_v11.py
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
Write-Host "PASS: v1.1 local verification completed"
Write-Host "Binary: .\target\release\yekaterina.exe"
Write-Host ""
Write-Host "Running the frozen v1.0.0 MCP golden correctness suite (expectations unchanged)..."
python .\golden\run_golden.py --exe ".\target\release\yekaterina.exe" --out ".\golden_results\latest"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "PASS: v1.1 local + MCP golden verification completed"
Write-Host "Golden report: .\golden_results\latest\REPORT.md"
Write-Host ""
Write-Host "Next:"
Write-Host "  1. .\RUN_FULL_CAPABILITY_AUDIT_WINDOWS.bat"
Write-Host "  2. .\RUN_BENCH_WINDOWS.bat   (compares against the frozen v1.0.0 baseline)"
