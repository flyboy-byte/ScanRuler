#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-only
"""Roadmap progress estimate from plan/roadmap.json and plan/units/*.json.

    python3 docs/port-plan/tools/roadmap.py --root docs/port-plan            # print
    python3 docs/port-plan/tools/roadmap.py --root docs/port-plan --write    # regenerate ROADMAP-PROGRESS.md
    python3 docs/port-plan/tools/roadmap.py --root docs/port-plan --check    # fail if invalid or stale

A piece with `units` is computed: the mean of each unit's status credit plus its verification
bonus (capped at 1). A piece with `estimate` is a dated judgement with a stated basis. Milestone
and overall figures are weight-averaged. These are progress estimates, not acceptance or parity.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import sys

OUT = "ROADMAP-PROGRESS.md"


def load(root: str) -> tuple[dict, dict]:
    with open(os.path.join(root, "plan", "roadmap.json"), encoding="utf-8") as f:
        roadmap = json.load(f)
    units = {}
    for path in sorted(glob.glob(os.path.join(root, "plan", "units", "*.json"))):
        with open(path, encoding="utf-8") as f:
            u = json.load(f)
        units[u["id"]] = u
    return roadmap, units


def is_phase(uid: str) -> bool:
    return len(uid) == 3 and uid[0] == "P" and uid[1:].isdigit()


def level(u: dict) -> str:
    v = u.get("verification", "none")
    return v if isinstance(v, str) else str(v.get("level", "none"))


def validate(roadmap: dict, units: dict) -> list[str]:
    errors: list[str] = []
    credit, bonus = roadmap["status_credit"], roadmap["verification_bonus"]
    placed: dict[str, str] = {}
    phases: dict[str, str] = {}
    ids: set[str] = set()
    for m in roadmap["milestones"]:
        if m.get("weight", 0) <= 0:
            errors.append(f"{m['id']}: weight must be positive")
        for p in m.get("phases", []):
            if p not in units:
                errors.append(f"{m['id']}: unknown phase {p}")
            if p in phases:
                errors.append(f"phase {p} in both {phases[p]} and {m['id']}")
            phases[p] = m["id"]
        if not m.get("pieces"):
            errors.append(f"{m['id']}: no pieces")
        for piece in m.get("pieces", []):
            pid = piece["id"]
            if pid in ids:
                errors.append(f"duplicate piece id {pid}")
            ids.add(pid)
            if piece.get("weight", 0) <= 0:
                errors.append(f"{pid}: weight must be positive")
            has_units, has_est = "units" in piece, "estimate" in piece
            if has_units == has_est:
                errors.append(f"{pid}: needs exactly one of units or estimate")
            if has_est:
                e = piece["estimate"]
                if not isinstance(e, (int, float)) or not 0 <= e <= 100:
                    errors.append(f"{pid}: estimate must be 0-100")
                if not piece.get("basis"):
                    errors.append(f"{pid}: estimate needs a basis")
            for uid in piece.get("units", []):
                if uid not in units:
                    errors.append(f"{pid}: unknown unit {uid}")
                    continue
                if is_phase(uid):
                    errors.append(f"{pid}: phase {uid} belongs in a milestone's phases, not a piece")
                if uid in placed:
                    errors.append(f"unit {uid} in both {placed[uid]} and {pid}")
                placed[uid] = pid
                if units[uid]["status"] not in credit:
                    errors.append(f"{uid}: status {units[uid]['status']} has no credit")
                if level(units[uid]) not in bonus:
                    errors.append(f"{uid}: verification {level(units[uid])} has no bonus")
    for uid in sorted(units):
        if is_phase(uid):
            if uid not in phases:
                errors.append(f"phase {uid} is in no milestone")
        elif uid not in placed:
            errors.append(f"unit {uid} is in no roadmap piece (add it to plan/roadmap.json)")
    return errors


def unit_credit(u: dict, roadmap: dict) -> float:
    c = roadmap["status_credit"][u["status"]] + roadmap["verification_bonus"][level(u)]
    return min(1.0, c)


def piece_pct(piece: dict, units: dict, roadmap: dict) -> float:
    if "estimate" in piece:
        return float(piece["estimate"])
    us = piece["units"]
    return 100.0 * sum(unit_credit(units[i], roadmap) for i in us) / len(us)


def weighted(pairs: list[tuple[float, float]]) -> float:
    total = sum(w for w, _ in pairs)
    return sum(w * v for w, v in pairs) / total if total else 0.0


def compute(roadmap: dict, units: dict) -> dict:
    out = []
    for m in roadmap["milestones"]:
        pieces = []
        for p in m["pieces"]:
            pieces.append({**p, "pct": piece_pct(p, units, roadmap)})
        pct = weighted([(p["weight"], p["pct"]) for p in pieces])
        measured = sum(p["weight"] for p in pieces if "units" in p) / sum(p["weight"] for p in pieces)
        out.append({**m, "pieces": pieces, "pct": pct, "measured": measured})
    overall = weighted([(m["weight"], m["pct"]) for m in out])
    return {"milestones": out, "overall": overall}


def bar(pct: float, width: int = 10) -> str:
    n = round(pct / 100 * width)
    return "█" * n + "░" * (width - n)


def render(roadmap: dict, units: dict) -> str:
    r = compute(roadmap, units)
    sc = ", ".join(f"{k} {v:g}" for k, v in roadmap["status_credit"].items())
    vb = ", ".join(f"{k} +{v:g}" for k, v in roadmap["verification_bonus"].items())
    lines = [
        "<!-- SPDX-License-Identifier: AGPL-3.0-only -->",
        "# Roadmap progress (estimate)",
        "",
        "Generated by `tools/roadmap.py` from `plan/roadmap.json` and the unit files; do not hand-edit.",
        f"Estimates as of **{roadmap['as_of']}**. Regenerate with "
        "`python3 docs/port-plan/tools/roadmap.py --root docs/port-plan --write`.",
        "",
        "> These are progress estimates, not acceptance or parity. No unit is accepted. *Computed* pieces",
        "> average their units' state credit; *estimate* pieces are dated judgements with a stated basis.",
        "> Weights are effort shares and are judgement too. Edit `plan/roadmap.json`, never this file.",
        "",
        f"**Overall: {r['overall']:.0f}%**",
        "",
        "| Milestone | Progress | % | Weight | Computed share |",
        "| --- | --- | ---: | ---: | ---: |",
    ]
    for m in r["milestones"]:
        lines.append(f"| {m['id']} {m['title']} | `{bar(m['pct'])}` | {m['pct']:.0f}% | {m['weight']} | {m['measured']:.0%} |")
    lines += ["", f"Unit credit: {sc}; verification bonus: {vb} (capped at 1).", ""]
    for m in r["milestones"]:
        lines += [f"## {m['id']} — {m['title']} ({m['pct']:.0f}%)", "", f"Phases: {', '.join(m['phases'])}.", ""]
        lines += ["| Piece | % | Weight | Basis | Left |", "| --- | ---: | ---: | --- | --- |"]
        for p in m["pieces"]:
            if "units" in p:
                done = sum(1 for i in p["units"] if units[i]["status"] in ("implemented", "integrated", "accepted"))
                basis = f"computed: {done}/{len(p['units'])} units implemented or better"
            else:
                basis = f"estimate: {p['basis']}"
            lines.append(f"| {p['title']} | {p['pct']:.0f}% | {p['weight']} | {basis} | {p.get('left', '')} |")
        lines.append("")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default="docs/port-plan")
    g = ap.add_mutually_exclusive_group()
    g.add_argument("--write", action="store_true", help=f"write {OUT}")
    g.add_argument("--check", action="store_true", help=f"fail if invalid or {OUT} is stale")
    a = ap.parse_args(argv)
    roadmap, units = load(a.root)
    errors = validate(roadmap, units)
    if errors:
        for e in errors:
            print(f"roadmap: {e}", file=sys.stderr)
        return 1
    text = render(roadmap, units)
    path = os.path.join(a.root, OUT)
    if a.write:
        with open(path, "w", encoding="utf-8") as f:
            f.write(text + "\n")
        print(f"wrote {path}")
    elif a.check:
        try:
            with open(path, encoding="utf-8") as f:
                current = f.read()
        except FileNotFoundError:
            current = ""
        if current != text + "\n":
            print(f"roadmap: {path} is stale; run tools/roadmap.py --write", file=sys.stderr)
            return 1
        print("roadmap: ok")
    else:
        print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
