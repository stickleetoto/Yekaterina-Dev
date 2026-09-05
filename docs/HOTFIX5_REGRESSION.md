# alpha.12-hotfix5 regression repair

This hotfix adds no opcodes and does not change the MCP tool/schema surface.

## Lambert W0

The hotfix4 principal-branch seed used the large-x approximation `ln(x)-ln(ln(x))` at `x >= 1`. At exactly `x=1`, `ln(1)=0`, so the seed becomes non-finite. Hotfix5 uses `ln1p(x)` for moderate positive inputs (`x < 3`) and keeps the asymptotic seed for larger inputs. The defining identity is now covered at `x=1` by both the Rust engine test and a live MCP Golden case.

## Fourier Nyquist regression

The hotfix4 test compared the complete floating-point coefficient array with exact JSON equality. On Windows the theoretically zero middle coefficient was `3.633735870479366e-17`, which is normal roundoff. Hotfix5 checks all three coefficients with `1e-12` tolerance while preserving the corrected Nyquist amplitude of `1.0`.

## Acceptance target

- 1,215 registered opcodes
- exactly 3 MCP tools
- 525/525 Golden cases across 44 categories
- Full Capability Audit 1,215/1,215 execution/type coverage
