"""Check cross-interface certificates, corpus freshness, and review trust boundaries."""
import copy
import json
from pathlib import Path
import sys
import unittest

import oathkeeper

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from benchmark import bundle, cases, evaluate
from compare_snapshots import compare
from review_packet import packet, validate_reviews
from lawlib_tools.runtime import module_paths

BASE = ROOT / ".deps/lawlib/us/wa/seattle"


class AssessmentWorkflowTests(unittest.TestCase):
    def test_real_record_analysis_and_independent_certificate_replay(self):
        context = bundle(BASE / "cases/long-excessive-fines.case.oath")
        analysis = json.loads(oathkeeper.analyze(json.dumps(context)))
        self.assertEqual(analysis["status"], "supported")
        self.assertNotIn("verifiability", analysis["supporting"][0]["report"])
        for argument in analysis["supporting"] + analysis["opposing"]:
            candidate = {**context, "proof": argument["proof"], "query": {**context["query"], "goal": argument["report"]["goal"]}}
            replay = json.loads(oathkeeper.verify(json.dumps(candidate), "json"))
            self.assertEqual(replay["bundle_sha256"], argument["report"]["bundle_sha256"])
        selected = next(s for s in analysis["sensitivity"] if s["id"] == "disproportionality-finding")
        self.assertEqual(selected["without"], "neither")

    def test_partial_record_and_abstaining_oracle_are_not_negative_findings(self):
        analysis = json.loads(oathkeeper.analyze_file(str(BASE / "cases/long-parking-evidence-review.case.oath")))
        self.assertEqual(analysis["status"], "neither")
        self.assertGreater(len(analysis["open_requirements"]), 0)
        oracle = json.loads(oathkeeper.analyze_file(str(BASE / "cases/long-oracle-reassessment.case.oath")))
        self.assertEqual(oracle["oracle_requests"][0]["status"], "pending")
        self.assertEqual(oracle["opposing"], [])

    def test_review_records_require_snapshot_and_independent_identified_reviewer(self):
        current = packet(BASE)
        review = json.loads(module_paths(ROOT / '.deps/lawlib', 'seattle')['review'].read_text())
        self.assertEqual(validate_reviews(review, current)["legal"], "pending")
        entry = review["records"][0]
        entry.update(status="approved", reviewer="synthetic-test-reviewer", independence_declared=True, rationale="Synthetic metadata test; not a real corpus approval.", recorded_at="2026-10-18T23:00:00Z")
        self.assertEqual(validate_reviews(review, current)[entry["role"]], "approval_reported")
        entry["module_snapshot_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "stale"):
            validate_reviews(review, current)
        entry["module_snapshot_sha256"] = current["module_snapshot_sha256"]
        entry["reviewer"] = review["authors"][0]
        with self.assertRaisesRegex(ValueError, "independence"):
            validate_reviews(review, current)
        entry["reviewer"] = None
        with self.assertRaisesRegex(ValueError, "identified"):
            validate_reviews(review, current)

    def test_new_unused_rule_requires_reanalysis_even_if_certificate_dependencies_unchanged(self):
        old = json.loads(oathkeeper.compile_file(str(BASE / "parking.oath")))["module"]
        report = json.loads(oathkeeper.verify_file(str(BASE / "cases/long-excessive-fines.case.oath")))
        new = copy.deepcopy(old)
        added = copy.deepcopy(new["rules"][0]);added["id"] = "synthetic-added-rule"
        new["rules"].append(added)
        impact = compare(old, new, [report])
        self.assertFalse(impact["proofs"][0]["requires_reverification"])
        self.assertTrue(impact["proofs"][0]["requires_reanalysis"])
        new["rules"][0]["premises"] = []
        self.assertTrue(compare(old, new, [report])["proofs"][0]["requires_reverification"])

    def test_deterministic_baseline_separates_real_and_synthetic_contexts(self):
        tasks = list(cases())
        self.assertTrue(all(t["context"]["proof"] == [] for t in tasks))
        self.assertEqual(len([t for t in tasks if t["origin"] == "synthetic_boundary_test"]), 4)
        result = evaluate()
        self.assertEqual(result["passed"], result["total"])
        self.assertEqual(result["independent_authentic_matters"], 1)


if __name__ == "__main__":
    unittest.main()
