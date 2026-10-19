"""A minimal reward evaluator: the model controls proof steps, never context."""
import argparse
import json
from pathlib import Path

import oathkeeper


def evaluate(context: dict, candidate: object, *, allow_oracles: bool = True) -> dict:
    # Host-side setup errors must not be mistaken for failed candidate proofs.
    oathkeeper.check_context(json.dumps(context, allow_nan=False))
    # An optional reference certificate validates task construction, but is never
    # required in the model-visible input and never copied into the candidate.
    if context.get("proof"):
        oathkeeper.verify(json.dumps(context, allow_nan=False), "json")
    if not isinstance(candidate, list):
        return {"reward": 0, "error": "candidate must be a proof array"}
    submitted = {**context, "proof": candidate}
    try:
        report = json.loads(oathkeeper.verify(json.dumps(submitted, allow_nan=False), "json"))
    except (ValueError, TypeError, OverflowError) as error:
        return {"reward": 0, "error": str(error)}
    accepted = allow_oracles or report["oracle_count"] == 0
    result = {"reward": int(accepted), "report": report}
    if not accepted:
        result["error"] = "this host task explicitly excludes oracle premises"
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("context", type=Path, help="host-controlled normalized JSON bundle")
    parser.add_argument("candidate", type=Path, help="candidate proof array as JSON")
    parser.add_argument("--exclude-oracles", action="store_true", help="optional task restriction; oracle count is never a reward bonus")
    args = parser.parse_args()
    context = json.loads(args.context.read_text())
    # Validate the trusted context even when the candidate is malformed.
    oathkeeper.check_context(json.dumps(context, allow_nan=False))
    try:
        candidate = json.loads(args.candidate.read_text())
    except ValueError as error:
        print(json.dumps({"reward": 0, "error": str(error)}))
        return
    print(json.dumps(evaluate(context, candidate, allow_oracles=not args.exclude_oracles), indent=2))


if __name__ == "__main__":
    main()
