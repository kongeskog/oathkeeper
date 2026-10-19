"""Build a review packet and validate review records without inventing reviewers."""
import argparse
import hashlib
import json
from datetime import datetime
from pathlib import Path

from .runtime import compile_source, DEFAULT


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()


def packet(base=DEFAULT):
    compilation = compile_source(base / "parking.oath")
    notes = json.loads((base / "interpretations.json").read_text())
    sources = json.loads((base / "sources.lock.json").read_text())["sources"]
    by_rule = {n["rule"]: n for n in notes["interpretations"]}
    rules = []
    for rule in compilation["module"]["rules"]:
        note = by_rule[rule["id"]]
        rules.append({
            "rule": rule["id"], "version": rule["version"], "rule_sha256": digest(rule),
            "source": rule["source"], "pinpoint": rule["authority"]["pinpoint"],
            "premises": rule["premises"], "conclusion": rule["conclusion"],
            "interpretation": note,
        })
    return {
        "format": "oathkeeper-review-packet/1",
        "module_snapshot_sha256": compilation["snapshot_sha256"],
        "rules": rules, "sources": sources,
        "review_questions": [
            "Is this the correct source edition and historical legal context?",
            "Does each formal premise and conclusion faithfully track the cited passage?",
            "Which relevant exceptions, definitions, authorities, and procedural conditions are omitted?",
            "Does a scope or interpretation premise hide the disputed legal issue?",
            "Does the formal derivation use the intended actors, matter, units, and time?",
            "Which alternative readings or counterarguments must remain visible?",
        ],
        "independent_review": "pending; generating this packet does not perform review",
        "digest_algorithm": "sha256 of UTF-8 JSON with sorted object keys and compact separators for rule_sha256; native compilation digest for module snapshot",
    }


def validate_reviews(review, current):
    if review.get("format") != "oathkeeper-review/2":
        raise ValueError("unsupported review manifest")
    records = review.get("records", [])
    keys = set()
    approved = set()
    authors = set(review.get("authors", []))
    for entry in records:
        key = entry["id"]
        if key in keys:
            raise ValueError("duplicate review record")
        keys.add(key)
        if entry["role"] not in {"legal", "formalization"}:
            raise ValueError("unknown review role")
        if entry["status"] not in {"pending", "approved", "changes_requested", "withdrawn"}:
            raise ValueError("unknown review status")
        if entry["status"] == "approved":
            if entry.get("module_snapshot_sha256") != current["module_snapshot_sha256"]:
                raise ValueError("approved review is stale for this exact module")
            reviewer = entry.get("reviewer")
            if not isinstance(reviewer, str) or not reviewer.strip() or reviewer in authors or entry.get("independence_declared") is not True:
                raise ValueError("approval requires an identified reviewer and declared independence from authors")
            if not entry.get("recorded_at") or not entry.get("rationale", "").strip():
                raise ValueError("approval requires a recorded time and rationale")
            if datetime.fromisoformat(entry["recorded_at"].replace("Z", "+00:00")).tzinfo is None:
                raise ValueError("review timestamp requires a timezone")
            approved.add(entry["role"])
    return {
        "legal": "approval_reported" if "legal" in approved else "pending",
        "formalization": "approval_reported" if "formalization" in approved else "pending",
        "identity_authentication": "not_performed; external reviewer identity and qualifications require human verification",
        "current_snapshot_sha256": current["module_snapshot_sha256"],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", nargs="?", type=Path, default=DEFAULT)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    current = packet(args.base)
    current["reviews"] = validate_reviews(json.loads((args.base / "review.json").read_text()), current)
    text = json.dumps(current, indent=2) + "\n"
    if args.output:
        args.output.write_text(text)
    else:
        print(text, end="")


if __name__ == "__main__":
    main()
