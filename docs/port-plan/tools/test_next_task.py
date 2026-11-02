import unittest
import next_task as nt

LANES = [{"id": "A", "units": ["a"], "name": "A", "touches": ""}, {"id": "B", "units": ["b", "c", "d"], "name": "B", "touches": ""}]


def u(i, status="planned", deps=(), parent=None, **k):
    return {"id": i, "title": i, "scope": "s", "acceptance": ["x"], "status": status, "depends_on": list(deps), "parent": parent, **k}


class Rank(unittest.TestCase):
    def test_ready_before_startable_before_prototype(self):
        units = [u("a", deps=["z"]), u("b"), u("c", deps=["e"]), u("e", "implemented"), u("z", "planned")]
        order = [r[2] for r in nt.rank(units, LANES, {})]
        self.assertEqual(order[0], "b")  # ready (no deps)
        self.assertLess(order.index("c"), order.index("a"))  # startable before prototype-only

    def test_claimed_and_parents_and_done_are_skipped(self):
        units = [u("a"), u("b", parent="a"), u("c", "accepted"), u("d")]
        ids = [r[2] for r in nt.rank(units, LANES, {"d": "pr"})]
        self.assertEqual(ids, ["b"])

    def test_claim_from_pr_text_is_exact_id(self):
        units = [u("P01"), u("P010")]
        got = nt.claimed_ids(units, [{"number": 7, "title": "P01: baseline", "headRefName": "x", "body": ""}])
        self.assertIn("P01", got)
        self.assertNotIn("P010", got)

    def test_claim_from_unit_file(self):
        self.assertIn("a", nt.claimed_ids([u("a", branch="codex/a")], [], trust_unit_fields=True))

    def test_stale_unit_field_is_ignored_when_prs_were_read(self):
        self.assertNotIn("a", nt.claimed_ids([u("a", branch="old/branch")], []))

    def test_unassigned_unit_is_last_and_visible(self):
        idx, lane = nt.lane_of("zzz", LANES)
        self.assertEqual(idx, len(LANES))
        self.assertIn("unassigned", lane["name"])


if __name__ == "__main__":
    unittest.main()
