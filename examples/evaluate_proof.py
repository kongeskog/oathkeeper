"""A minimal reward evaluator: the model controls proof steps, never context."""
import argparse
import json
from pathlib import Path

import oathkeeper


def evaluate(context: dict, candidate: object, *, allow_oracles: bool = True, require_coverage: bool = False) -> dict:
    # These arguments are host-owned. Candidate data is confined to proof steps.
    task = json.loads(oathkeeper.assess_policy(json.dumps({
        "operation": "freeze", "bundle": context,
        "policy": {"allow_oracles": allow_oracles, "require_coverage": require_coverage},
    }, allow_nan=False)))
    expected = task["task_sha256"]  # Store separately in a service; never take it from a candidate.
    return json.loads(oathkeeper.assess_policy(json.dumps({
        "operation": "evaluate", "task": task, "expected_sha256": expected, "candidate": candidate,
    }, allow_nan=False)))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("context", type=Path, help="host-controlled normalized JSON bundle")
    parser.add_argument("candidate", type=Path, help="candidate proof array as JSON")
    parser.add_argument("--exclude-oracles", action="store_true", help="optional task restriction; oracle count is never a reward bonus")
    parser.add_argument("--require-coverage", action="store_true", help="require a proof for every selected scope condition")
    args = parser.parse_args()
    context = json.loads(args.context.read_text())
    # Validate the trusted context even when the candidate is malformed.
    oathkeeper.check_context(json.dumps(context, allow_nan=False))
    try:
        candidate = json.loads(args.candidate.read_text())
    except ValueError as error:
        print(json.dumps({"reward": 0, "error": str(error)}))
        return
    print(json.dumps(evaluate(context, candidate, allow_oracles=not args.exclude_oracles, require_coverage=args.require_coverage), indent=2))


if __name__ == "__main__":
    main()
