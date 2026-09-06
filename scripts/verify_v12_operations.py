"""Check every v1.2 operation against an independently computed value.

Run:  python scripts/verify_v12_operations.py
Needs a release binary at target/release/yekaterina.exe.


The expectations here are written from the definition, not from the Rust: where
the engine builds a closed form, this recomputes it by a different route
(iteration, or the inverse operation) so a shared mistake is unlikely.
"""
from __future__ import annotations
import math, sys, json, tempfile
from decimal import Decimal, ROUND_HALF_UP, ROUND_HALF_EVEN, ROUND_FLOOR, ROUND_CEILING, ROUND_DOWN, getcontext
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "bench"))
from bench_client import BenchClient, mcp_text  # noqa: E402

# Windows and CI (Linux) name the binary differently; pick whichever exists.
_exe = ROOT / "target/release/yekaterina.exe"
EXE = str(_exe if _exe.exists() else ROOT / "target/release/yekaterina")

getcontext().prec = 200

# Numbers chosen to be past the point where a float or an i64 stops being exact:
# 2**53 is where float64 loses adjacent integers, 2**63 is where i64 ends.
BIG = 2 ** 128 + 1
HUGE = 7 ** 300


def trunc_divmod(a, b):
    """Truncated division -- Python's // is floor, so it cannot be used here."""
    q = abs(a) // abs(b)
    if (a < 0) != (b < 0):
        q = -q
    return q, a - q * b


def pmt(p, annual_pct, months):
    r = annual_pct / 100 / 12
    if r == 0:
        return p / months
    q = (1 + r) ** months
    return p * r * q / (q - 1)


def balance_iterative(p, annual_pct, months, paid):
    """Amortise month by month -- deliberately not the closed form the engine uses."""
    r = annual_pct / 100 / 12
    m = pmt(p, annual_pct, months)
    bal = p
    for _ in range(paid):
        bal = bal + bal * r - m
    return bal


def npv(rate_pct, flows):
    r = rate_pct / 100
    return sum(cf / (1 + r) ** t for t, cf in enumerate(flows))


def bond(face, coupon_pct, market_pct, n):
    """Sum the coupons one by one rather than using the annuity closed form."""
    c = face * coupon_pct / 100
    y = market_pct / 100
    return sum(c / (1 + y) ** t for t in range(1, int(n) + 1)) + face / (1 + y) ** n


SQ3 = math.sqrt(3)

CASES = [
    # op, args, expected
    ("pct.increase", [200, 15], 230.0),
    ("pct.decrease", [200, 15], 170.0),
    ("pct.ratio", [45, 180], 25.0),
    ("pct.reverse", [230, 15], 200.0),
    ("pct.point_change", [3.5, 4.25], 0.75),
    ("pct.apply", [100, [10, -10]], 99.0),

    ("fin.npv", [10, [-1000, 400, 400, 400]], npv(10, [-1000, 400, 400, 400])),
    ("fin.annuity_pv", [1000, 5, 10], sum(1000 / 1.05 ** t for t in range(1, 11))),
    ("fin.annuity_fv", [1000, 5, 10], sum(1000 * 1.05 ** t for t in range(10))),
    ("fin.perpetuity_pv", [1000, 5], 20000.0),
    ("fin.loan_balance", [300000, 6, 360, 60], balance_iterative(300000, 6, 360, 60)),
    ("fin.loan_total_interest", [300000, 6, 360], pmt(300000, 6, 360) * 360 - 300000),
    ("fin.amort_interest", [300000, 6, 360, 1], 300000 * 0.005),
    ("fin.amort_principal", [300000, 6, 360, 1], pmt(300000, 6, 360) - 1500.0),
    ("fin.depreciation_straight", [10000, 2000, 5], 1600.0),
    ("fin.depreciation_declining", [10000, 2000, 5, 1], 4000.0),
    ("fin.depreciation_declining", [10000, 2000, 5, 4], 8000 - (4000 + 2400 + 1440)),
    ("fin.depreciation_syd", [10000, 2000, 5, 1], 8000 * 5 / 15),
    ("fin.depreciation_syd", [10000, 2000, 5, 5], 8000 * 1 / 15),
    ("fin.break_even_units", [10000, 25, 15], 1000.0),
    ("fin.margin", [60, 100], 40.0),
    ("fin.markup", [60, 100], 200 / 3),
    ("fin.effective_rate", [12, 12], (1.01 ** 12 - 1) * 100),
    ("fin.nominal_rate", [(1.01 ** 12 - 1) * 100, 12], 12.0),
    ("fin.rule72", [8], 9.0),
    ("fin.payback_period", [1000, [400, 400, 400]], 2.5),
    ("fin.bond_price", [1000, 5, 6, 10], bond(1000, 5, 6, 10)),
    ("fin.real_rate", [7, 3], (1.07 / 1.03 - 1) * 100),

    ("unit.force", [1, "kN", "N"], 1000.0),
    ("unit.force", [1, "kgf", "N"], 9.80665),
    ("unit.torque", [1, "kNm", "Nm"], 1000.0),
    ("unit.density", [1, "gcm3", "kgm3"], 1000.0),
    ("unit.flow", [1, "m3s", "Ls"], 1000.0),
    ("unit.flow", [60, "Lmin", "Ls"], 1.0),
    ("unit.acceleration", [1, "g", "mps2"], 9.80665),
    ("unit.charge", [1, "Ah", "C"], 3600.0),
    ("unit.charge", [1000, "mAh", "Ah"], 1.0),
    ("unit.illuminance", [1, "phot", "lux"], 10000.0),

    ("geo.midpoint3d", [[0, 0, 0], [2, 4, 6]], [1.0, 2.0, 3.0]),
    ("geo.cylinder_volume", [2, 5], math.pi * 4 * 5),
    ("geo.cylinder_area", [2, 5], 2 * math.pi * 2 * (2 + 5)),
    ("geo.cone_volume", [3, 4], math.pi * 9 * 4 / 3),
    ("geo.cone_area", [3, 4], math.pi * 3 * (3 + 5)),          # slant height 5 (3-4-5)
    ("geo.cube_volume", [3], 27.0),
    ("geo.cube_area", [3], 54.0),
    ("geo.box_volume", [2, 3, 4], 24.0),
    ("geo.box_area", [2, 3, 4], 52.0),
    ("geo.pyramid_volume", [12, 5], 20.0),
    ("geo.torus_volume", [5, 2], 2 * math.pi ** 2 * 5 * 4),
    ("geo.torus_area", [5, 2], 4 * math.pi ** 2 * 5 * 2),
    ("geo.ellipse_area", [3, 2], math.pi * 6),
    ("geo.ellipse_area", [3, 3], math.pi * 9),                  # degenerates to a circle
    ("geo.ellipse_perimeter", [3, 3], 2 * math.pi * 3),         # circle: exact check
    ("geo.trapezoid_area", [3, 5, 4], 16.0),
    ("geo.parallelogram_area", [6, 4], 24.0),
    ("geo.regular_polygon_area", [6, 2], 6 * SQ3),              # 6 unit-side triangles
    ("geo.regular_polygon_area", [4, 2], 4.0),                  # a square
    ("geo.regular_polygon_perimeter", [6, 2], 12.0),
    ("geo.circle_sector_area", [3, math.tau], math.pi * 9),     # full turn = whole circle
    ("geo.circle_sector_area", [3, math.pi / 2], math.pi * 9 / 4),
    ("geo.circle_segment_area", [3, math.pi], math.pi * 9 / 2), # half turn = semicircle
    ("geo.point_line_distance", [[0, 3], [0, 0], [4, 0]], 3.0),
    ("geo.triangle_area_points", [[0, 0], [4, 0], [0, 3]], 6.0),

    ("vec.normalize", [[3, 4]], [0.6, 0.8]),
    ("vec.angle", [[1, 0], [0, 1]], math.pi / 2),
    ("vec.angle", [[1, 1], [1, 0]], math.pi / 4),
    ("vec.reject", [[3, 4], [1, 0]], [0.0, 4.0]),
    ("vec.reflect", [[1, -1], [0, 1]], [1.0, 1.0]),
    ("vec.lerp", [[0, 0], [10, 20], 0.25], [2.5, 5.0]),
    ("vec.manhattan", [[1, 2, 3], [4, 6, 3]], 7.0),
    ("vec.chebyshev", [[1, 2, 3], [4, 6, 3]], 4.0),
    ("vec.minkowski", [[1, 2, 3], [4, 6, 3], 3], 91 ** (1 / 3)),
    ("vec.minkowski", [[1, 2, 3], [4, 6, 3], 1], 7.0),          # p=1 must equal manhattan
    ("vec.minkowski", [[1, 2, 3], [4, 6, 3], 2], 5.0),          # p=2 must equal euclidean
    ("vec.hadamard", [[1, 2, 3], [4, 5, 6]], [4.0, 10.0, 18.0]),
    ("vec.negate", [[1, -2, 3]], [-1.0, 2.0, -3.0]),
    ("vec.abs", [[1, -2, 3]], [1.0, 2.0, 3.0]),
    ("vec.triple3", [[1, 0, 0], [0, 1, 0], [0, 0, 1]], 1.0),
    ("vec.triple3", [[2, 0, 0], [0, 3, 0], [0, 0, 4]], 24.0),
    ("vec.rotate2d", [[1, 0], math.pi / 2], [0.0, 1.0]),
    ("vec.rotate2d", [[1, 0], math.pi], [-1.0, 0.0]),
]


def _s(x):
    return str(x)


# (op, args, expected) where expected is compared as an exact integer or decimal,
# not as a string, so a formatting choice cannot make a wrong value look right.
EXACT_CASES = [
    ("int.abs", [_s(-BIG)], abs(-BIG)),
    ("int.neg", [_s(BIG)], -BIG),
    ("int.sign", [_s(-BIG)], -1),
    ("int.sign", ["0"], 0),
    ("int.cmp", [_s(2**53 + 1), _s(2**53)], 1),
    ("int.cmp", [_s(BIG), _s(BIG)], 0),
    ("int.div_floor", ["-7", "2"], -7 // 2),
    ("int.mod_floor", ["-7", "2"], -7 % 2),
    ("int.div_floor", [_s(-HUGE), "97"], -HUGE // 97),
    ("int.mod_floor", [_s(-HUGE), "97"], -HUGE % 97),
    ("int.sqrt", [_s(HUGE)], math.isqrt(HUGE)),
    ("int.sqrt", [_s(10**30)], 10**15),
    ("int.shl", ["1", 128], 1 << 128),
    ("int.shr", [_s(1 << 128), 128], 1),
    ("int.shr", [_s(-(1 << 128)), 3], -(1 << 128) >> 3),
    ("int.bit_length", [_s(1 << 128)], (1 << 128).bit_length()),
    ("int.bit_length", ["0"], 0),
    ("int.min", [[_s(BIG), "-5", _s(HUGE)]], -5),
    ("int.max", [[_s(BIG), "-5", _s(HUGE)]], HUGE),
    ("int.sum", [[_s(2**53), "1", _s(2**53)]], 2**53 + 1 + 2**53),
    ("int.product", [["99999999999"] * 3], 99999999999 ** 3),
    ("int.mod_pow", ["2", "1000", "1000000007"], pow(2, 1000, 1000000007)),
    ("int.mod_inverse", ["3", "1000000007"], pow(3, -1, 1000000007)),
    # The point of the arbitrary-precision versions: a 2048-bit modulus, where
    # alg.mod_pow (u64) cannot even accept the arguments.
    ("int.mod_pow", [_s(HUGE), _s(65537), _s(2**2048 - 1809251394333065553493296640760748560207343510400633813116524750123642650623)],
     pow(HUGE, 65537, 2**2048 - 1809251394333065553493296640760748560207343510400633813116524750123642650623)),

    ("dec.mod", ["10.5", "3"], Decimal("10.5") % Decimal("3")),
    ("dec.abs", ["-12.750"], Decimal("12.75")),
    ("dec.neg", ["12.75"], Decimal("-12.75")),
    ("dec.cmp", ["0.1", "0.10"], 0),
    ("dec.cmp", ["0.2", "0.1"], 1),
    # 2.675 is exactly representable as a decimal and is not as a float; the
    # exact answer is 2.68 where a float-based round gives 2.67.
    ("dec.round", ["2.675", 2], Decimal("2.675").quantize(Decimal("0.01"), ROUND_HALF_UP)),
    ("dec.round", ["1234.5", -2], Decimal("1200")),
    ("dec.round", ["-2.5", 0], Decimal("-2.5").quantize(Decimal("1"), ROUND_HALF_UP)),
    ("dec.round_even", ["2.5", 0], Decimal("2.5").quantize(Decimal("1"), ROUND_HALF_EVEN)),
    ("dec.round_even", ["3.5", 0], Decimal("3.5").quantize(Decimal("1"), ROUND_HALF_EVEN)),
    ("dec.floor", ["-2.1"], Decimal("-2.1").quantize(Decimal("1"), ROUND_FLOOR)),
    ("dec.ceil", ["-2.1"], Decimal("-2.1").quantize(Decimal("1"), ROUND_CEILING)),
    ("dec.trunc", ["-2.9"], Decimal("-2.9").quantize(Decimal("1"), ROUND_DOWN)),
    ("dec.scale", ["1.2300"], 4),
    ("dec.scale", ["5"], 0),
    ("dec.min", [["0.1", "0.02", "0.3"]], Decimal("0.02")),
    ("dec.max", [["0.1", "0.02", "0.3"]], Decimal("0.3")),
    # The headline case: 0.1 + 0.2 is exactly 0.3 here, and 0.30000000000000004
    # in float64.
    ("dec.sum", [["0.1", "0.2"]], Decimal("0.3")),
    ("dec.sum", [["0.1"] * 10], Decimal("1")),
    ("dec.product", [["0.1", "0.2", "0.3"]], Decimal("0.006")),
    ("dec.pow", ["1.05", 20], Decimal("1.05") ** 20),
    ("dec.pow", ["2", 0], Decimal("1")),
]

EXACT_ERRORS = [
    ("int.divmod", ["1", "0"], "DIV0"),
    ("int.div_floor", ["1", "0"], "DIV0"),
    ("int.sqrt", ["-1"], "DOMAIN"),
    ("int.mod_pow", ["2", "-1", "7"], "DOMAIN"),
    ("int.mod_pow", ["2", "3", "0"], "DIV0"),
    # 4 and 8 share a factor, so no inverse exists.
    ("int.mod_inverse", ["4", "8"], "DOMAIN"),
    ("int.mod_inverse", ["3", "-7"], "DOMAIN"),
    ("int.shl", ["1", 99999999], "LIMIT"),
    ("int.sum", [[]], "EMPTY"),
    ("int.min", ["nope"], "TYPE"),
    ("dec.mod", ["1", "0"], "DIV0"),
    ("dec.sum", [[]], "EMPTY"),
    # A negative exponent cannot terminate exactly, so it is refused rather
    # than silently approximated.
    ("dec.pow", ["2", -1], "TYPE"),
    ("dec.round", ["1", 999999], "LIMIT"),
    # 1e400 is a perfectly good decimal; it is the float that cannot hold it,
    # so this is NONFINITE (conversion overflowed) and not TYPE (bad input).
    ("dec.to_number", ["1e400"], "NONFINITE"),
    ("dec.to_number", ["not a number"], "TYPE"),
]

# Errors that must be produced, so the guards are tested too.
ERRORS = [
    ("geo.regular_polygon_area", [2, 1], "DOMAIN"),        # fewer than 3 sides
    ("geo.regular_polygon_area", [6.5, 1], "DOMAIN"),      # fractional side count
    ("geo.torus_volume", [2, 5], "DOMAIN"),                # tube thicker than the hole
    ("geo.cube_volume", [-1], "DOMAIN"),
    ("geo.circle_sector_area", [1, 7], "DOMAIN"),          # past a full turn
    ("geo.point_line_distance", [[0, 1], [2, 2], [2, 2]], "DEGENERATE"),
    ("vec.normalize", [[0, 0]], "DOMAIN"),
    ("vec.angle", [[0, 0], [1, 0]], "DOMAIN"),
    ("vec.minkowski", [[1], [2], 0.5], "DOMAIN"),          # p below 1 is not a metric
    ("vec.rotate2d", [[1, 2, 3], 1], "SHAPE"),
    ("vec.triple3", [[1, 0], [0, 1], [0, 0]], "SHAPE"),
    ("fin.irr", [[100, 200]], "NO_CONVERGE"),              # no sign change, no root
    ("fin.rule72", [0], "DOMAIN"),
    ("fin.perpetuity_pv", [100, 0], "DOMAIN"),
    ("fin.break_even_units", [100, 10, 10], "DOMAIN"),     # zero contribution margin
    ("fin.amort_interest", [1000, 5, 12, 13], "DOMAIN"),   # period past the term
    ("fin.depreciation_straight", [100, 200, 5], "DOMAIN"),# salvage above cost
    ("fin.loan_balance", [1000, 5, 12, 13], "DOMAIN"),
    ("unit.force", [1, "m", "N"], "UNIT"),
    ("pct.ratio", [1, 0], "DIV0"),
]


def close(got, exp, tol=1e-9):
    if isinstance(exp, list):
        return (isinstance(got, list) and len(got) == len(exp)
                and all(close(g, e, tol) for g, e in zip(got, exp)))
    return isinstance(got, (int, float)) and math.isclose(float(got), float(exp), rel_tol=tol, abs_tol=1e-12)


def main():
    bad, checked = [], 0
    with tempfile.TemporaryDirectory(prefix="yk-v12-") as home:
        with BenchClient(EXE, env={"YEKATERINA_HOME": home}, timeout=120) as c:
            for op, a, exp in CASES:
                p = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                checked += 1
                if "r" not in p or not close(p["r"], exp):
                    bad.append(f"VALUE {op}{a}: got {p} want {exp}")
            for op, a, exp in EXACT_CASES:
                p = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                checked += 1
                if "r" not in p:
                    bad.append(f"EXACT {op}{a}: got {p} want {exp}")
                    continue
                got = p["r"]
                if isinstance(got, str) and ("e" in got or "E" in got):
                    bad.append(f"FORMAT {op}{a}: exact result uses exponent notation: {got}")
                    continue
                try:
                    ok = (int(got) == exp) if isinstance(exp, int) else (Decimal(str(got)) == exp)
                except Exception:
                    ok = False
                if not ok:
                    bad.append(f"EXACT {op}{a}: got {got!r} want {exp}")
            for op, a, code in list(ERRORS) + list(EXACT_ERRORS):
                p = json.loads(mcp_text(c.tool_call("yk.compute", {"op": op, "a": a}).response))
                checked += 1
                if p.get("e") != code:
                    bad.append(f"ERROR {op}{a}: got {p} want {{'e': '{code}'}}")
            # IRR is checked by its definition: NPV at the returned rate is zero.
            p = json.loads(mcp_text(c.tool_call(
                "yk.compute", {"op": "fin.irr", "a": [[-1000, 400, 400, 400]]}).response))
            checked += 1
            if "r" not in p or abs(npv(p["r"], [-1000, 400, 400, 400])) > 1e-6:
                bad.append(f"IRR does not zero the NPV: {p}")

    families = set(o for o, *_ in CASES) | set(o for o, *_ in EXACT_CASES)
    print(f"checked {checked} assertions across {len(families)} operations")
    if bad:
        print(f"\n{len(bad)} FAILURES:")
        for b in bad:
            print("  " + b)
        raise SystemExit(1)
    print("PASS: every v1.2 operation matches an independently computed value")


if __name__ == "__main__":
    main()
