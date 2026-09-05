# Yekaterina alpha.8 Operation Inventory

Alpha.8 contains **524 built-in/control opcodes** while exposing exactly **3 MCP tools**.

## Major families

| Family | Count | Purpose |
|---|---:|---|
| stat | 60 | descriptive/robust/error statistics |
| prob | 55 | distributions, discrete probability, Bayes, information measures |
| mat | 48 | dense matrix arithmetic and decompositions |
| signal | 45 | sequence processing, transforms and spectra |
| math | 40 | scalar scientific mathematics |
| alg | 36 | exact algebra, number theory, combinatorics, polynomials |
| eng | 36 | engineering formulas |
| num | 36 | numerical methods |
| cplx | 34 | complex-number mathematics |
| phys | 34 | physics formulas |
| reg | 14 | regression |
| unit | 13 | unit conversion |
| bit | 11 | bit operations |
| test | 10 | hypothesis/effect-size statistics |
| vec | 9 | vectors |
| geo | 8 | geometry |
| fin | 8 | finance |
| int | 8 | exact integer arithmetic |
| udo | 7 | user-defined operation lifecycle |
| base | 4 | radix conversion |
| dec | 4 | arbitrary precision decimals |
| pct | 3 | percentage helpers |
| expr | 1 | safe one-off arithmetic expressions |

Run `python scripts/operation_manifest.py` to regenerate the authoritative count from `src/registry.rs`.

## Important design rule

The operation inventory is internal. None of these operations is surfaced as an individual MCP tool. LLM clients discover only what they need through `yk.find` and `yk.spec`, then execute through `yk.compute`.
