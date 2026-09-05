# Yekaterina v0.1.0-alpha.5 — Operation Expansion

Alpha.5 expands the internal operation registry from 48 to 116 registered built-in/control opcodes while preserving the three-tool MCP surface.

## New families / coverage

- advanced scalar math: inverse/hyperbolic trig, log base, degree/radian conversion, lerp, approximate equality
- statistics: percentile/quartiles/IQR, mode, geometric/harmonic mean, RMS, MAD, sample stats, covariance/correlation, z-score, normalization, cumulative sum
- vectors: add/sub/scale/dot/norm/distance/cosine/cross3
- geometry: 2D/3D distance, midpoint, circle/rectangle/triangle calculations
- practical percentage and finance calculations
- unit conversion: length, mass, temperature, data units
- signal basics: difference, moving average, EMA, convolution

## Architecture invariant

External MCP tools remain exactly:

- `yk.compute`
- `yk.find`
- `yk.spec`

Adding operations must not add new MCP tools.

## Measured alpha.3 proof baseline (regression target)

The previous MCP-only real-world benchmark measured:

- Arithma tools: 174
- Yekaterina tools: 3
- schema tokens: 16,896 vs 412 (97.56% reduction)
- 10,000 arithmetic operations: 812,100 vs 159,794 wire tokens (80.32% reduction)
- MCP call reduction at 10k: 99.90%
- tested arithmetic accuracy: 100% for both
- recovery and UDO persistence: PASS

These values are protocol measurements, not LLM billing-token claims.

Alpha.5 must rerun the same benchmark after compilation. The target is to preserve the three-tool surface and keep schema-token growth negligible despite the registry growing past 100 operations.
