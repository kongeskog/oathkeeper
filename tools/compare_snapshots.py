"""Describe changed declarations and which supplied proof reports need replay."""
import argparse
import json
from pathlib import Path


def compare(old, new, reports=()):
    old = old.get("module", old)
    new = new.get("module", new)
    changed = []
    changed_rules, changed_authorities, changed_assessments = set(), set(), set()
    for collection, target in [("rules", changed_rules), ("authorities", changed_authorities), ("authority_assessments", changed_assessments)]:
        left = {item["id"]: item for item in old.get(collection, [])}
        right = {item["id"]: item for item in new.get(collection, [])}
        for identifier in sorted(left.keys() | right.keys()):
            if left.get(identifier) != right.get(identifier):
                target.add(identifier)
                a, b = left.get(identifier), right.get(identifier)
                kind = "added" if a is None else "removed" if b is None else "changed"
                changed.append({"collection": collection, "id": identifier, "change": kind,
                    "source_artifact_changed": bool(a and b and a.get("source") != b.get("source"))})
    global_changes = [key for key in ("language", "types", "predicates", "constants", "jurisdictions") if old.get(key) != new.get(key)]
    affected = []
    for report in reports:
        reasons = list(global_changes)
        reasons += ["rule:" + r[0] for r in report["rules_used"] if r[0] in changed_rules]
        reasons += ["authority:" + a["id"] for a in report["authorities_used"] if a["id"] in changed_authorities]
        reasons += ["assessment:" + a["id"] for a in report["authority_assessments_used"] if a["id"] in changed_assessments]
        affected.append({"bundle_sha256": report["bundle_sha256"], "goal": report["goal"], "requires_reverification": bool(reasons), "requires_reanalysis": bool(changed or global_changes), "reasons": reasons})
    return {"format": "oathkeeper-change-impact/1", "declarations": changed, "global_changes": global_changes, "proofs": affected,
        "meaning": "Dependency impact only. This does not determine whether a change reflects enacted law, corrected encoding, or a new interpretation; review must classify that cause. A new counterargument may also require reassessment of previously supported goals."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("old", type=Path)
    parser.add_argument("new", type=Path)
    parser.add_argument("--report", type=Path, action="append", default=[])
    args = parser.parse_args()
    print(json.dumps(compare(json.loads(args.old.read_text()), json.loads(args.new.read_text()), [json.loads(p.read_text()) for p in args.report]), indent=2))


if __name__ == "__main__":
    main()
