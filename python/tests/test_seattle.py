import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import sys

import oathkeeper

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "lawlib" / "us" / "wa" / "seattle"
sys.path.insert(0, str(ROOT / "tools"))
SPEC = importlib.util.spec_from_file_location("audit_lawlib", ROOT / "tools" / "audit_lawlib.py")
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
REPLAY_SPEC = importlib.util.spec_from_file_location("refetch_sources", ROOT / "tools" / "refetch_sources.py")
REPLAY = importlib.util.module_from_spec(REPLAY_SPEC)
REPLAY_SPEC.loader.exec_module(REPLAY)


class SeattleTests(unittest.TestCase):
    def test_real_law_native_api(self):
        report = json.loads(oathkeeper.verify_case(
            (BASE / "parking.module.json").read_text(),
            (BASE / "cases" / "long-excessive-fines.case.json").read_text(),
        ))
        self.assertEqual(len(report["authorities_used"]), 3)
        self.assertEqual(report["unreviewed_authority_assessment_count"], 5)
        self.assertEqual(report["goal"]["arguments"][0]["value"], "long.98824-2")

    def test_source_and_coverage_inventory(self):
        report = AUDIT.audit(BASE, proofs=False)
        self.assertEqual(report["sources_verified"], 4)
        self.assertEqual(report["formalized_rules"], 14)
        self.assertFalse(report["coverage_complete"])

    def test_tampered_source_fails_audit(self):
        # Synthetic tiny lock; the actual corpus is never modified.
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            lock = {"format": "oathkeeper-source-lock/1", "sources": [{"id": "test", "file": "test.txt", "sha256": "0" * 64}]}
            (base / "sources.lock.json").write_text(json.dumps(lock))
            (base / "test.txt").write_text("changed bytes")
            with self.assertRaisesRegex(ValueError, "source bytes changed"):
                AUDIT.audit_sources(base)

    def test_extraction_excludes_editorial_markup(self):
        html = b"<h1>Editorial summary</h1><noframes>Judicial &amp; text\n only.</noframes><footer>Ads</footer>"
        self.assertEqual(REPLAY.extract(html, "html_noframes_normalized"), b"Judicial & text only.\n")
        with self.assertRaises(ValueError):
            REPLAY.extract(b"<p>Missing opinion</p>", "html_noframes_normalized")


if __name__ == "__main__":
    unittest.main()
