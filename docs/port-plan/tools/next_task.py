#!/usr/bin/env python3
"""Pick the next unit of work from the plan, so a session needs no prompt or handoff.

Reads plan/units/*.json and plan/lanes.json; optionally asks `gh` which units open PRs already claim.
Not a parity tool and it changes nothing: it only ranks and prints.
"""
import argparse
import json
import re
import subprocess
from pathlib import Path

WORK = {"planned", "specified", "baseline-captured"}
DONE_ENOUGH = {"baseline-captured", "implemented", "compared", "integrated", "accepted"}
CLASS_NAME = {0: "ready", 1: "startable", 2: "prototype-only"}


def classify(unit, by_id):
    """ready: hard deps accepted. startable: deps have a real result. prototype-only: some dep is still planned."""
    statuses = [by_id[d]["status"] for d in unit.get("depends_on", []) if d in by_id]
    if all(s == "accepted" for s in statuses):
        return 0
    if all(s in DONE_ENOUGH for s in statuses):
        return 1
    return 2


def lane_of(unit_id, lanes):
    for index, lane in enumerate(lanes):
        if unit_id in lane["units"]:
            return index, lane
    return len(lanes), {"id": "?", "name": "unassigned (add to plan/lanes.json)", "touches": ""}


def claimed_ids(units, prs, trust_unit_fields=False):
    """Unit ids named in an open PR title or branch. Unit-file claim fields count only when the open
    PRs could not be read, because a branch recorded in a unit file outlives its PR."""
    found = {}
    for unit in units:
        if trust_unit_fields and (unit.get("claim_pr") or unit.get("branch")):
            found[unit["id"]] = f"unit file: branch={unit.get('branch')} pr={unit.get('claim_pr')}"
    for pr in prs:
        text = " ".join(str(pr.get(k, "")) for k in ("title", "headRefName"))
        for unit in units:
            if re.search(r"(?<![\w.])" + re.escape(unit["id"]) + r"(?![\w.])", text, re.I):
                found.setdefault(unit["id"], f"open PR #{pr.get('number')} {pr.get('title', '')}")
    return found


def rank(units, lanes, claimed):
    parents = {u.get("parent") for u in units if u.get("parent")}
    by_id = {u["id"]: u for u in units}
    rows = []
    for unit in units:
        if unit["status"] not in WORK or unit["id"] in parents or unit["id"] in claimed:
            continue
        lane_index, lane = lane_of(unit["id"], lanes)
        rows.append((classify(unit, by_id), lane_index, unit["id"], unit, lane))
    rows.sort(key=lambda r: r[:3])
    return rows


def open_prs():
    try:
        out = subprocess.run(
            ["gh", "pr", "list", "-R", "flyboy-byte/scanruler", "--state", "open", "--limit", "100",
             "--json", "number,title,headRefName"],
            capture_output=True, text=True, timeout=30, check=True).stdout
        return json.loads(out), None
    except Exception as error:  # gh missing, offline, or unauthenticated
        return [], f"could not read open PRs ({type(error).__name__}); claims below come from unit files only"


def show(row, by_id):
    cls, _, _, unit, lane = row
    print(f"\n[{CLASS_NAME[cls]}] {unit['id']}  {unit['title']}   (lane {lane['id']}: {lane['name']})")
    deps = unit.get("depends_on", [])
    print("  depends on: " + (", ".join(f"{d}={by_id[d]['status']}[{by_id[d].get('verification', 'none')}]" for d in deps if d in by_id) or "nothing"))
    print(f"  verification: {unit.get('verification', 'none')}")
    print(f"  scope: {unit['scope']}")
    for item in unit["acceptance"]:
        print(f"  accept: {item}")
    if unit.get("source_paths"):
        print("  original source: " + ", ".join(unit["source_paths"][:8]) + (" ..." if len(unit["source_paths"]) > 8 else ""))
    for item in unit.get("open_items", []):
        print(f"  open: {item}")
    if lane["touches"]:
        print(f"  likely touches: {lane['touches']}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("-n", type=int, default=3, help="how many to show")
    parser.add_argument("--offline", action="store_true", help="do not ask gh for open PRs")
    args = parser.parse_args()
    units = [json.loads(p.read_text()) for p in sorted((args.root / "plan/units").glob("*.json"))]
    lanes = json.loads((args.root / "plan/lanes.json").read_text())["lanes"]
    by_id = {u["id"]: u for u in units}
    prs, warning = ([], None) if args.offline else open_prs()
    claimed = claimed_ids(units, prs, trust_unit_fields=bool(warning) or args.offline)
    rows = rank(units, lanes, claimed)
    print(f"{len(units)} units; {len(rows)} unclaimed leaf units need work; {len(claimed)} claimed.")
    if warning:
        print("WARNING: " + warning)
    unassigned = [u["id"] for u in units if lane_of(u["id"], lanes)[0] == len(lanes) and u["status"] in WORK]
    if unassigned:
        print("Not in any lane (add to plan/lanes.json): " + ", ".join(unassigned[:12]) + (" ..." if len(unassigned) > 12 else ""))
    for uid, why in sorted(claimed.items()):
        print(f"claimed: {uid} <- {why}")
    for row in rows[: args.n]:
        show(row, by_id)
    if not rows:
        print("Nothing unclaimed and actionable: report that instead of inventing work.")
    else:
        print("\nTake the first one. Claim it by opening a draft PR whose title starts with the unit id.")


if __name__ == "__main__":
    main()
