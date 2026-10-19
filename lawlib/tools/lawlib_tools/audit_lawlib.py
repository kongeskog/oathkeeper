"""Check pinned source bytes, provenance references, and every listed legal proof."""
import argparse
import hashlib
import json
from pathlib import Path
import oathkeeper
from . import sources as evidence_store
from .runtime import ROOT, DEFAULT, compile_source

def audit_sources(base: Path) -> dict:
    base = base.resolve()
    lock = json.loads((base / "sources.lock.json").read_text())
    if lock["format"] != "oathkeeper.sources/2":
        raise ValueError("unsupported source lock format")
    by_digest = {}
    ids = set()
    for entry in lock["sources"]:
        if entry["id"] in ids:
            raise ValueError(f"duplicate source ID: {entry['id']}")
        ids.add(entry["id"])
        evidence_store.require(entry)
        actual = entry["sha256"]
        if actual in by_digest:
            raise ValueError("duplicate source digest in lock")
        by_digest[actual] = entry
    return by_digest


def check_references(value: object, sources: dict) -> None:
    if isinstance(value, dict):
        if "source" in value:
            source = value["source"]
            if source["sha256"] not in sources:
                raise ValueError(f"unpinned source: {source['citation']}")
            if sources[source["sha256"]]["uri"] != source["uri"]:
                raise ValueError(f"source URI does not match lock: {source['citation']}")
        if "evidence_sha256" in value and value["evidence_sha256"] not in sources:
            raise ValueError("unpinned evidence digest")
        for item in value.values():
            check_references(item, sources)
    elif isinstance(value, list):
        for item in value:
            check_references(item, sources)


def audit(base: Path, *, proofs: bool = True) -> dict:
    sources = audit_sources(base)
    coverage = json.loads((base / "coverage.json").read_text())
    module_path = base / coverage["module"]
    authoring = module_path.suffix == ".oath"
    compilation = compile_source(module_path) if authoring else None
    module = compilation["module"] if authoring else json.loads(module_path.read_text())
    check_references(module, sources)
    rule_ids = [r["id"] for r in module["rules"]]
    if len(set(rule_ids)) != len(rule_ids) or set(rule_ids) != set(coverage["rule_ids"]):
        raise ValueError("coverage rule inventory does not match module")
    checked = []
    for case_name in coverage["cases"]:
        case_path = base / case_name
        case = compile_source(case_path)["case"] if authoring else json.loads(case_path.read_text())
        check_references(case, sources)
        if proofs:
            report = json.loads(oathkeeper.verify_file(str(case_path)))
            checked.append({"case": case_name, "goal": report["goal"], "rules_used": len(report["rules_used"]), "unreviewed_assessments": report["unreviewed_authority_assessment_count"]})
    pending = []
    for case_name in coverage.get("pending_cases", []):
        case_path = base / case_name
        check_references(compile_source(case_path)["case"], sources)
        if proofs:
            try:
                oathkeeper.verify_file(str(case_path))
            except ValueError as error:
                if "E_MISSING_EVIDENCE" not in str(error): raise
            else:
                raise ValueError("pending application unexpectedly verified")
        pending.append(case_name)
    assessed = []
    for case_name in coverage.get("assessment_cases", []):
        case_path = base / case_name
        check_references(compile_source(case_path)["case"], sources)
        if proofs:
            result = json.loads(oathkeeper.analyze_file(str(case_path)))
            if result["status"] != "neither" or not result["search"]["complete_for_ground_domain"]:
                raise ValueError("record-only pilot must expose unresolved requirements, not a fabricated legal result")
            assessed.append({"case":case_name,"status":result["status"],"open_requirements":len(result["open_requirements"])})
    reviews = None
    if coverage.get("format") == "oathkeeper-coverage/3":
        from .review_packet import packet, validate_reviews
        current = packet(base)
        reviews = validate_reviews(json.loads((base / "review.json").read_text()), current)
    return {
        "sources_verified": len(sources), "formalized_rules": len(rule_ids),
        "cases_verified": checked, "coverage_complete": coverage["coverage_complete"],
        "review_status": coverage["review_status"],
        "pending_oracle_applications": pending,
        "imported_source_files": len(compilation["lock"]) if compilation else 0,
        "record_assessments": assessed,
        "reviews": reviews,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", nargs="?", type=Path, default=DEFAULT)
    parser.add_argument("--sources-only", action="store_true", help="skip Rust proof execution")
    args = parser.parse_args()
    print(json.dumps(audit(args.base.resolve(), proofs=not args.sources_only), indent=2))


if __name__ == "__main__":
    main()
