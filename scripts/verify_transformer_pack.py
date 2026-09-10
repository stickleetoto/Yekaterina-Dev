#!/usr/bin/env python3
from __future__ import annotations

import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK_PATH = ROOT / "packs" / "xfmr_design_v1.json"

FORBIDDEN_DUPLICATE_SEMANTICS = {
    "transformer_voltage",
    "transformer_current",
    "transformer_impedance",
    "resistance_from_resistivity",
    "current_density",
    "copper_loss",
    "efficiency",
}


def close(actual: float, expected: float, *, rel: float = 1e-11, abs_: float = 1e-12) -> None:
    if not math.isclose(actual, expected, rel_tol=rel, abs_tol=abs_):
        raise AssertionError(f"{actual!r} != {expected!r}")


def eval_formula(expr: str, params: list[str], args: list[float]) -> float:
    env = dict(zip(params, args, strict=True))
    py_expr = expr.replace("^", "**")
    value = eval(py_expr, {"__builtins__": {}}, env)
    if not isinstance(value, (int, float)) or not math.isfinite(float(value)):
        raise AssertionError(f"non-finite formula result: {expr}")
    return float(value)


def main() -> None:
    pack = json.loads(PACK_PATH.read_text(encoding="utf-8"))
    assert pack["v"] == 1
    assert pack["name"] == "xfmr"
    ops = pack["ops"]
    assert len(ops) == 20

    by_name = {}
    for item in ops:
        assert item["k"] == "f"
        op = item["op"]
        assert op.startswith("pack.xfmr.")
        assert op not in by_name
        tail = op.removeprefix("pack.xfmr.")
        assert tail not in FORBIDDEN_DUPLICATE_SEMANTICS
        assert item["p"]
        assert len(item["p"]) == len(set(item["p"]))
        by_name[op] = item

    def run(tail: str, args: list[float]) -> float:
        item = by_name[f"pack.xfmr.{tail}"]
        return eval_formula(item["expr"], item["p"], args)

    gross_area = 0.020
    stacking = 0.95
    ae = gross_area * stacking
    close(run("core_effective_area", [gross_area, stacking]), ae)

    f = 60.0
    bmax = 1.55
    volts_per_turn = 4.44 * f * bmax * ae
    close(run("volts_per_turn_sine", [f, bmax, ae]), volts_per_turn)

    hv = 13_200.0
    turns = hv / volts_per_turn
    close(run("turns_sine", [hv, f, bmax, ae]), turns)
    close(run("flux_density_sine", [hv, f, turns, ae]), bmax)

    bsat = 1.90
    close(run("saturation_margin_pct", [bsat, bmax]), (bsat - bmax) / bsat * 100.0)

    mu0 = 4.0 * math.pi * 1e-7
    path = 1.25
    mur = 1500.0
    reluctance = path / (mu0 * mur * ae)
    close(run("magnetic_reluctance", [path, mur, ae]), reluctance)
    lm = mu0 * mur * turns**2 * ae / path
    close(run("magnetizing_inductance", [turns, mur, ae, path]), lm)

    h = 85.0
    close(run("magnetizing_current_from_h", [h, path, turns]), h * path / turns)

    current = 120.28130608117205
    j = 2.5
    close(run("conductor_area_mm2", [current, j]), current / j)

    mlt = 0.72
    close(run("winding_length", [turns, mlt]), turns * mlt)

    r20 = 0.18
    alpha_cu = 0.00393
    temp_c = 95.0
    close(
        run("resistance_at_temperature", [r20, alpha_cu, temp_c]),
        r20 * (1.0 + alpha_cu * (temp_c - 20.0)),
    )

    k = 0.0021
    alpha = 1.47
    beta = 2.12
    loss_density = k * f**alpha * bmax**beta
    close(run("core_loss_density_steinmetz", [k, f, alpha, bmax, beta]), loss_density)

    core_volume = 0.043
    close(run("core_loss_from_density", [loss_density, core_volume]), loss_density * core_volume)

    density = 7650.0
    core_mass = density * core_volume
    close(run("core_mass", [density, core_volume]), core_mass)
    close(run("core_loss_from_specific_loss", [1.05, core_mass]), 1.05 * core_mass)

    close(run("window_fill_pct", [3150.0, 9000.0]), 35.0)

    s_va = 100_000.0
    v_line = 480.0
    rated_i = s_va / (math.sqrt(3.0) * v_line)
    close(run("rated_current_three_phase", [s_va, v_line]), rated_i)

    r_eq = 0.018
    x_eq = 0.042
    pf = 0.90
    sin_phi = math.sqrt(1.0 - pf**2)
    lag = rated_i * (r_eq * pf + x_eq * sin_phi) / v_line * 100.0
    lead = rated_i * (r_eq * pf - x_eq * sin_phi) / v_line * 100.0
    z_pct = rated_i * math.sqrt(r_eq**2 + x_eq**2) / v_line * 100.0
    close(run("voltage_regulation_lagging_pct", [rated_i, r_eq, x_eq, pf, v_line]), lag)
    close(run("voltage_regulation_leading_pct", [rated_i, r_eq, x_eq, pf, v_line]), lead)
    close(run("short_circuit_impedance_pct", [rated_i, r_eq, x_eq, v_line]), z_pct)

    print("TRANSFORMER PACK VERIFY PASS")
    print(f"operations={len(ops)}")
    print(f"sample_hv_turns={turns:.6f}")
    print(f"sample_lv_rated_current_a={rated_i:.6f}")


if __name__ == "__main__":
    main()
