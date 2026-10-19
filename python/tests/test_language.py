import json
from pathlib import Path
import unittest

import oathkeeper

BASE = Path(__file__).resolve().parents[2] / ".deps/lawlib/us/wa/seattle"


class LanguageTests(unittest.TestCase):
    def test_compilation_and_native_verification(self):
        path = str(BASE / "cases/long-excessive-fines.case.oath")
        artifact = json.loads(oathkeeper.compile_file(path))
        report = json.loads(oathkeeper.verify_file(path))
        self.assertEqual(artifact["module"]["language"], "oathkeeper/0.5")
        self.assertEqual(len(artifact["lock"]), 5)
        self.assertEqual(len(artifact["module"]["rules"]), 14)
        self.assertGreater(report["judicial_finding_count"], 0)
        self.assertEqual(report["assurance"]["legal_fidelity"], "not_established_by_kernel")
        via_json = json.loads(oathkeeper.verify_case(json.dumps(artifact["module"]), json.dumps(artifact["case"])))
        self.assertEqual(via_json["bundle_sha256"], report["bundle_sha256"])

    def test_json_cannot_bypass_domain_types(self):
        artifact = json.loads(oathkeeper.compile_file(str(BASE / "cases/long-excessive-fines.case.oath")))
        artifact["case"]["constants"][0]["sort"] = {"domain": "Vehicle"}
        with self.assertRaisesRegex(ValueError, "E_TYPE"):
            oathkeeper.verify_case(json.dumps(artifact["module"]), json.dumps(artifact["case"]))

    def test_missing_oracle_is_not_a_verified_result(self):
        with self.assertRaisesRegex(ValueError, r"\.oath:\d+:\d+:.*E_MISSING_EVIDENCE"):
            oathkeeper.verify_file(str(BASE / "cases/long-oracle-reassessment.case.oath"))


if __name__ == "__main__":
    unittest.main()
