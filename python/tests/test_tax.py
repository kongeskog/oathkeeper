import json
import unittest
from pathlib import Path
import oathkeeper

BASE = Path(__file__).resolve().parents[2] / '.deps/lawlib/us/tax'
SCENARIOS = Path(__file__).resolve().parents[2] / '.deps/lawlib/tests/scenarios/us/tax'

class TaxBindingTests(unittest.TestCase):
    def test_allocation_certificate_and_real_case_replay(self):
        result = json.loads(oathkeeper.assess_backdoor_roth((SCENARIOS/'synthetic-pro-rata.json').read_text()))
        self.assertEqual(result['allocation']['taxable_conversion_cents'], 630000)
        replay = json.loads(oathkeeper.verify(json.dumps(result['certificate']), 'json'))
        self.assertEqual(replay, result['verification'])
        self.assertEqual(json.loads(oathkeeper.verify_file(str(BASE/'shank-2018.case.oath')))['status'], 'derivation_verified')

    def test_unknown_basis_and_invalid_request(self):
        scenario = json.loads((SCENARIOS/'synthetic-clean.json').read_text())
        scenario['prior_basis_cents'] = None
        result = json.loads(oathkeeper.assess_backdoor_roth(json.dumps(scenario)))
        self.assertEqual(result['status'], 'needs_information')
        self.assertIsNone(result['certificate'])
        scenario['prior_basis_cents'] = 1.5
        with self.assertRaises(ValueError):
            oathkeeper.assess_backdoor_roth(json.dumps(scenario))
