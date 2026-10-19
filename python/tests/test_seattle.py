import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import os
from unittest.mock import patch
import sys

import oathkeeper

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / ".deps/lawlib" / "us" / "wa" / "seattle"
sys.path.insert(0, str(ROOT / "tools"))
SPEC = importlib.util.spec_from_file_location("audit_lawlib", ROOT / "tools" / "audit_lawlib.py")
AUDIT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(AUDIT)
REPLAY_SPEC = importlib.util.spec_from_file_location("refetch_sources", ROOT / "tools" / "refetch_sources.py")
REPLAY = importlib.util.module_from_spec(REPLAY_SPEC)
REPLAY_SPEC.loader.exec_module(REPLAY)


class SeattleTests(unittest.TestCase):
    def test_real_law_native_api(self):
        compiled = json.loads(oathkeeper.compile_file(str(BASE / "cases/long-excessive-fines.case.oath")))
        report = json.loads(oathkeeper.verify_case(json.dumps(compiled['module']), json.dumps(compiled['case'])))
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
            entry = {"id":"test", "file":"test.txt", "sha256":"0"*64, "size_bytes":13,
                     "media_type":"text/plain", "mirrors":["https://example.invalid/test.txt"]}
            lock = {"format":"oathkeeper.sources/2", "sources":[entry]}
            (base / "sources.lock.json").write_text(json.dumps(lock))
            with patch.dict(os.environ, {"OATHKEEPER_CACHE_DIR":str(base/'cache')}):
                path=AUDIT.evidence_store.object_path(entry)
                path.parent.mkdir(parents=True)
                path.write_bytes(b"changed bytes")
                with self.assertRaisesRegex(ValueError, "source bytes changed"):
                    AUDIT.audit_sources(base / 'sources.lock.json')

    def test_extraction_excludes_editorial_markup(self):
        html = b"<h1>Editorial summary</h1><noframes>Judicial &amp; text\n only.</noframes><footer>Ads</footer>"
        self.assertEqual(REPLAY.extract(html, "html_noframes_normalized"), b"Judicial & text only.\n")
        with self.assertRaises(ValueError):
            REPLAY.extract(b"<p>Missing opinion</p>", "html_noframes_normalized")


if __name__ == "__main__":
    unittest.main()
