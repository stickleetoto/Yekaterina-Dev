# Alpha.9 Operation Expansion

Alpha.9 adds 185 operations and raises the registry from 524 to 709 built-in/control opcodes without adding MCP tools.

## New families

- `disc.*` — 30: exact combinatorics, Bell/Stirling/Lah numbers, discrete sequences, digit/Collatz helpers
- `chem.*` — 32: mole/mass/particle conversion, concentration, ideal gas, acid/base, spectrophotometry, colligative calculations
- `net.*` — 33: IPv4/CIDR addressing plus network timing/capacity/fragmentation calculations
- `color.*` — 27: RGB/HSV/HSL/CMYK/YIQ/YCbCr transforms, luminance, contrast and alpha/color adjustments
- `info.*` — 30: Shannon metrics, KL/JS divergence, channel/coding formulas, string distance and small checksums
- `astro.*` — 33: gravity/orbits, black-hole radius, photometry, Hohmann transfers and photon/radiation helpers

## Exact-output convention

Large discrete/combinatorial integer results use decimal strings, matching the existing exact BigInt convention.

## Registry scalability

Alpha.9 exact opcode/alias resolution uses a lazy `OnceLock<HashMap<String, usize>>`. Fuzzy `yk.find` remains bounded linear search because it is an explicitly requested discovery path, not the hot execution path.
