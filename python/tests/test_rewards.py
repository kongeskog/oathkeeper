"""Exercise the boundary between model submissions and host-controlled context."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("evaluate_proof", ROOT / "examples" / "evaluate_proof.py")
EVALUATOR = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EVALUATOR)


class RewardTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        normalized = subprocess.check_output(
            ["cargo", "run", "--quiet", "--locked", "--", "normalize", "examples/hoa.oath"],
            cwd=ROOT,
            text=True,
        )
        cls.context = json.loads(normalized)

    def test_agreed_inputs_receive_no_oracle_count_penalty(self):
        proof = self.context["proof"]
        self.assertEqual(EVALUATOR.evaluate(self.context, proof, allow_oracles=False)["reward"], 0)
        self.assertEqual(EVALUATOR.evaluate(self.context, proof, allow_oracles=True)["reward"], 1)

    def test_reference_certificate_is_optional_and_not_leaked_into_candidate(self):
        context = {**self.context, "proof": []}
        self.assertEqual(EVALUATOR.evaluate(context, self.context["proof"])["reward"], 1)
        self.assertEqual(EVALUATOR.evaluate(context, [])["reward"], 0)

    def test_model_cannot_replace_context(self):
        fake_context = {**self.context, "facts": []}
        self.assertEqual(EVALUATOR.evaluate(self.context, fake_context, allow_oracles=True)["reward"], 0)

    def test_forged_proof_receives_zero(self):
        proof = copy.deepcopy(self.context["proof"])
        proof[-1]["reason"] = {"kind": "fact", "fact": "notice"}
        self.assertEqual(EVALUATOR.evaluate(self.context, proof, allow_oracles=True)["reward"], 0)

    def test_invalid_context_is_a_host_error(self):
        context = {**self.context, "oracle_policy": []}
        with self.assertRaises(ValueError):
            EVALUATOR.evaluate(context, self.context["proof"], allow_oracles=True)

    def test_evaluation_leaves_context_intact(self):
        before = copy.deepcopy(self.context)
        EVALUATOR.evaluate(self.context, [], allow_oracles=True)
        self.assertEqual(self.context, before)


if __name__ == "__main__":
    unittest.main()
