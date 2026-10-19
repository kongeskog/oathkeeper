"""Check pinned source bytes, provenance references, and every listed legal proof."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DEFAULT = ROOT / "lawlib" / "us" / "wa" / "seattle"


def compile_source(path: Path) -> dict:
    try:
        import oathkeeper
    except ImportError:
        oathkeeper = None
    if oathkeeper is not None and hasattr(oathkeeper, "compile_file"):
        return json.loads(oathkeeper.compile_file(str(path)))
    return json.loads(subprocess.check_output(
        ["cargo", "run", "--quiet", "--locked", "--", "compile", str(path)],
        cwd=ROOT, text=True,
    ))


def audit_sources(base: Path) -> dict:
    base = base.resolve()
    lock = json.loads((base / "sources.lock.json").read_text())
    if lock["format"] != "oathkeeper-source-lock/1":
        raise ValueError("unsupported source lock format")
    by_digest = {}
    ids = set()
    for entry in lock["sources"]:
        if entry["id"] in ids:
            raise ValueError(f"duplicate source ID: {entry['id']}")
        ids.add(entry["id"])
        path = (base / entry["file"]).resolve()
        if not path.is_relative_to(base):
            raise ValueError("source path escapes corpus directory")
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != entry["sha256"]:
            raise ValueError(f"source bytes changed: {entry['file']}")
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
    if authoring:
        lock = json.loads((base / coverage["import_lock"]).read_text())
        if lock != {"format": "oathkeeper-import-lock/1", "snapshot_sha256": compilation["snapshot_sha256"], "sources": compilation["lock"]}:
            raise ValueError("compiled source graph does not match pinned import lock")
    if coverage.get("format") == "oathkeeper-coverage/3":
        if json.loads((base / "parking.module.json").read_text()) != module:
            raise ValueError("compiled JSON module is stale; run tools/export_lawlib.py")
        for source_path in sorted((base / "cases").glob("*.case.oath")):
            if json.loads(source_path.with_suffix(".json").read_text()) != compile_source(source_path)["case"]:
                raise ValueError(f"compiled JSON case is stale: {source_path.name}")
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
            output = subprocess.check_output(
                ["cargo", "run", "--quiet", "--locked", "--"] +
                (["verify", str(case_path)] if authoring else ["verify-case", str(module_path), str(case_path)]),
                cwd=ROOT, text=True,
            )
            report = json.loads(output)
            checked.append({"case": case_name, "goal": report["goal"], "rules_used": len(report["rules_used"]), "unreviewed_assessments": report["unreviewed_authority_assessment_count"]})
    pending = []
    for case_name in coverage.get("pending_cases", []):
        case_path = base / case_name
        check_references(compile_source(case_path)["case"], sources)
        if proofs:
            result = subprocess.run(["cargo", "run", "--quiet", "--locked", "--", "verify", str(case_path)], cwd=ROOT, text=True, capture_output=True)
            if result.returncode == 0 or "E_MISSING_EVIDENCE" not in result.stderr:
                raise ValueError(f"pending application no longer fails for missing evidence: {case_name}")
        pending.append(case_name)
    assessed = []
    for case_name in coverage.get("assessment_cases", []):
        case_path = base / case_name
        check_references(compile_source(case_path)["case"], sources)
        if proofs:
            result = json.loads(subprocess.check_output(["cargo", "run", "--quiet", "--locked", "--", "analyze", str(case_path)], cwd=ROOT, text=True))
            if result["status"] != "neither" or not result["search"]["complete_for_ground_domain"]:
                raise ValueError("record-only pilot must expose unresolved requirements, not a fabricated legal result")
            assessed.append({"case":case_name,"status":result["status"],"open_requirements":len(result["open_requirements"])})
    reviews = None
    if coverage.get("format") == "oathkeeper-coverage/3":
        from review_packet import packet, validate_reviews
        current = packet(base)
        reviews = validate_reviews(json.loads((base / "review.json").read_text()), current)
        saved = json.loads((base / "review-packet.json").read_text())
        expected = {**current, "reviews": reviews}
        if saved != expected:
            raise ValueError("review packet is stale; regenerate it without inventing review decisions")
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
