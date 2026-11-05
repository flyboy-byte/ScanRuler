# SPDX-License-Identifier: AGPL-3.0-only
import os
import unittest

import roadmap as rm

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")


def base():
    return {
        "as_of": "2026-11-04",
        "status_credit": {"planned": 0.0, "implemented": 0.75, "accepted": 1.0},
        "verification_bonus": {"none": 0.0, "fuzzed": 0.1},
        "milestones": [
            {"id": "M1", "title": "t", "weight": 1, "phases": ["P00"], "pieces": [
                {"id": "a", "title": "a", "weight": 1, "units": ["x", "y"]},
                {"id": "b", "title": "b", "weight": 3, "estimate": 20, "basis": "why"},
            ]},
        ],
    }


def units(**status):
    out = {"P00": {"id": "P00", "status": "planned", "verification": "none"}}
    for k, v in status.items():
        out[k] = {"id": k, "status": v, "verification": "none"}
    return out


class Roadmap(unittest.TestCase):
    def test_computed_and_estimate_are_weight_averaged(self):
        r = rm.compute(base(), units(x="implemented", y="accepted"))
        m = r["milestones"][0]
        self.assertAlmostEqual(m["pieces"][0]["pct"], 87.5)
        self.assertAlmostEqual(m["pct"], (87.5 + 3 * 20) / 4)
        self.assertAlmostEqual(m["measured"], 0.25)

    def test_bonus_is_capped(self):
        u = units(x="accepted", y="accepted")
        u["x"]["verification"] = "fuzzed"
        self.assertEqual(rm.compute(base(), u)["milestones"][0]["pieces"][0]["pct"], 100.0)

    def test_unplaced_unit_fails(self):
        errs = rm.validate(base(), units(x="planned", y="planned", z="planned"))
        self.assertTrue(any("unit z is in no roadmap piece" in e for e in errs))

    def test_double_placement_and_missing_basis_fail(self):
        r = base()
        r["milestones"][0]["pieces"][1] = {"id": "b", "title": "b", "weight": 1, "units": ["x"]}
        r["milestones"][0]["pieces"].append({"id": "c", "title": "c", "weight": 1, "estimate": 5})
        errs = rm.validate(r, units(x="planned", y="planned"))
        self.assertTrue(any("unit x in both" in e for e in errs))
        self.assertTrue(any("c: estimate needs a basis" in e for e in errs))

    def test_phase_outside_milestones_fails(self):
        u = units(x="planned", y="planned")
        u["P01"] = {"id": "P01", "status": "planned", "verification": "none"}
        self.assertIn("phase P01 is in no milestone", rm.validate(base(), u))

    def test_repository_roadmap_is_valid_and_current(self):
        self.assertEqual(rm.main(["--root", ROOT, "--check"]), 0)


if __name__ == "__main__":
    unittest.main()
