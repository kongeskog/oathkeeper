"""Run after `maturin develop`: python -m unittest discover -s python/tests."""
import json
from pathlib import Path
import unittest

import oathkeeper

SOURCE = (Path(__file__).resolve().parents[2] / "examples" / "hoa.oath").read_text()


class BindingTests(unittest.TestCase):
    def test_verified_report(self):
        report = json.loads(oathkeeper.verify(SOURCE))
        self.assertEqual(report["status"], "derivation_verified")
        self.assertEqual(report["oracle_count"], 1)
        self.assertEqual(report["oracle_assumptions"], ["color-assessment-1"])

    def test_rejection_is_an_exception(self):
        with self.assertRaises(ValueError):
            oathkeeper.verify(SOURCE.replace('confidence = 0.92', 'confidence = 0.01'))

    def test_bad_format(self):
        with self.assertRaises(ValueError):
            oathkeeper.verify(SOURCE, "yaml")


if __name__ == "__main__":
    unittest.main()
