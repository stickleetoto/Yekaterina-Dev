# alpha.12-hotfix6 registry identity repair

## Issue

`data.throughput_bps` (bits per second) and `data.throughput_Bps` (bytes per second) are intentionally distinct canonical opcodes. The alpha.12 registry lowercased every lookup key, so both collapsed onto the first registered opcode. This caused `yk.spec("data.throughput_Bps")` to return the `data.throughput_bps` spec and could also route `yk.compute` to the bit/s implementation, producing an 8x unit error.

## Repair

Resolution now has two levels:

1. exact case-sensitive canonical/alias lookup,
2. existing case-insensitive fallback for compatibility and convenient aliases.

`yk.find` uses the same exact-identity preference for exact queries.

## Regression coverage

- exact canonical resolution for both opcodes,
- exact alias resolution for both aliases,
- exact `yk.find` preference,
- engine computation proving `800 bytes / 2 s = 400 Bps` and `3200 bps`,
- two MCP Golden cases.

A full case-fold collision scan of all 1,215 canonical opcodes found no other canonical collision.
