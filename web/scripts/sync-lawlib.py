"""Refresh application artifacts from the actual native Oathkeeper compiler/kernel."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

import oathkeeper

ROOT = Path(__file__).resolve().parents[1]
CORE = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT.parent
BASE = CORE / "lawlib/us/wa/seattle"
ASSETS = ROOT / "public/assets"

def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")

compilation = json.loads(oathkeeper.compile_file(str(BASE / "parking.oath")))
module = compilation["module"]
save(ROOT / "src/data/module.json", module)
save(ASSETS / "parking.module.json", module)
for name in ["coverage.json", "sources.lock.json", "oath.lock.json", "review.json", "review-packet.json", "interpretations.json", "pilot.json", "baseline.json", "migration-0.4.json"]:
    shutil.copyfile(BASE / name, ASSETS / name)
    shutil.copyfile(BASE / name, ROOT / "src/data" / name)
records = json.loads((ROOT / "src/data/cases.json").read_text())
case_sources = {}
for record in records:
    slug = record["slug"]
    path = BASE / "cases" / f"{slug}.case.oath"
    compiled = json.loads(oathkeeper.compile_file(str(path)))
    bundle = {**compiled["module"], **compiled["case"]}
    bundle["constants"] = compiled["module"].get("constants", []) + compiled["case"].get("constants", [])
    report = json.loads(oathkeeper.verify_file(str(path)))
    assert json.loads(oathkeeper.verify(json.dumps(bundle), "json"))["bundle_sha256"] == report["bundle_sha256"]
    record.update(bundle=bundle, report=report)
    save(ASSETS / f"{slug}.bundle.json", bundle)
    save(ASSETS / f"{slug}.report.json", report)
    save(ASSETS / f"{slug}.case.json", compiled["case"])
    shutil.copyfile(path, ASSETS / path.name)
    case_sources[slug] = path.read_text()
save(ROOT / "src/data/cases.json", records)
save(ROOT / "src/data/case-sources.json", case_sources)
rule_sources = {}
for span in compilation["source_map"]:
    if span["kind"] != "rule":
        continue
    path = Path(span["file"])
    rule_sources[span["id"]] = {"file": str(path.relative_to(CORE)), "line": span["line"], "code": "\n".join(path.read_text().splitlines()[span["line"]-1:span["end_line"]])}
save(ROOT / "src/data/rule-sources.json", rule_sources)
for name, path in {
    **{name+".rs":"crates/oathkeeper-core/src/"+name+".rs" for name in ["contracts","evaluation","receipts","policy"]},
    "analysis.rs":"crates/oathkeeper-core/src/analysis.rs", "oracles.rs":"crates/oathkeeper-core/src/oracles.rs",
    "wasm.rs":"crates/oathkeeper-wasm/src/lib.rs", "parking-assessment.oath":"lawlib/us/wa/seattle/cases/long-parking-evidence-review.case.oath",
    "core.rs":"crates/oathkeeper-core/src/lib.rs", "model.rs":"crates/oathkeeper-core/src/model.rs",
    "kernel.rs":"crates/oathkeeper-core/src/kernel.rs", "schema.rs":"crates/oathkeeper-core/src/schema.rs",
    "compiler.rs":"crates/oathkeeper-core/src/compiler/mod.rs", "syntax.rs":"crates/oathkeeper-core/src/compiler/syntax.rs",
    "arguments.rs":"crates/oathkeeper-core/src/arguments.rs", "cli.rs":"crates/oathkeeper-cli/src/main.rs",
    "python.rs":"crates/oathkeeper-python/src/lib.rs", "source-seattle.oath":"lawlib/us/wa/seattle/parking.oath",
    "source-washington.oath":"lawlib/us/wa/long.oath", "source-federal.oath":"lawlib/us/timbs.oath",
    "vocabulary.oath":"lawlib/vocabulary.oath", "pending-oracle.oath":"lawlib/us/wa/seattle/cases/long-oracle-reassessment.case.oath",
}.items():
    shutil.copyfile(CORE / path, ROOT / "src/reference" / name)
    shutil.copyfile(CORE / path, ASSETS / name)
for name, path in {
    "SPEC-0.5.md":"docs/SPEC-0.5.md", "SPEC-0.4.md":"docs/SPEC-0.4.md", "RESEARCH.md":"docs/RESEARCH.md", "SOUNDNESS.md":"docs/SOUNDNESS.md",
    "GETTING_STARTED.md":"README.md", "SPEC-0.3.md":"docs/SPEC-0.3.md", "SPEC-0.2.md":"docs/SPEC-0.2.md",
    "ARCHITECTURE.md":"docs/ARCHITECTURE.md", "TRAINING.md":"docs/TRAINING.md", "SEATTLE.md":"lawlib/us/wa/seattle/README.md",
    "SEATTLE_PLAN.md":"docs/SEATTLE_PLAN.md", "ROADMAP.md":"docs/ROADMAP.md",
}.items():
    shutil.copyfile(CORE / path, ROOT / "src/docs" / name)
    shutil.copyfile(CORE / path, ASSETS / name)
workbench = []
for path in sorted((BASE / "cases").glob("*.case.oath")):
    compiled = json.loads(oathkeeper.compile_file(str(path)))
    bundle = {**compiled["module"], **compiled["case"]}
    bundle["constants"] = compiled["module"].get("constants", []) + compiled["case"].get("constants", [])
    bundle["proof"] = []
    slug = path.name.replace(".case.oath", "")
    title = next((r["title"] for r in records if r["slug"] == slug),
        "Parking evidence gaps" if "parking-evidence" in slug else "Pending subjective assessment")
    baseline = json.loads(oathkeeper.analyze(json.dumps(bundle)))
    workbench.append({"slug": slug, "title": title, "bundle": bundle, "baseline": baseline})
    save(ASSETS / f"{slug}.context.json", bundle)
    shutil.copyfile(path, ASSETS / path.name)
save(ROOT / "src/data/workbench-cases.json", workbench)
for source in json.loads((BASE / "sources.lock.json").read_text())["sources"]:
    shutil.copyfile(BASE / source["file"], ASSETS / Path(source["file"]).name)
# The federal pilot is a separate versioned module, never merged into Seattle's scope.
TAX = CORE / "lawlib/us/tax"
tax_assets = ASSETS / "tax"
tax_assets.mkdir(exist_ok=True)
for path in TAX.rglob("*"):
    if path.is_file():
        target = tax_assets / path.relative_to(TAX)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
tax_compilation = json.loads(oathkeeper.compile_file(str(TAX / "backdoor-roth-2025.oath")))
tax_rules = {}
for span in tax_compilation["source_map"]:
    if span["kind"] == "rule":
        path = Path(span["file"])
        tax_rules[span["id"]] = "\n".join(path.read_text().splitlines()[span["line"]-1:span["end_line"]])
tax_cases = []
for name in ["shank-2018", "shank-unavailable-returns"]:
    c = json.loads(oathkeeper.compile_file(str(TAX / (name + ".case.oath"))))
    b = {**c["module"], **c["case"]}
    b["constants"] = c["module"].get("constants", []) + c["case"].get("constants", [])
    report = json.loads(oathkeeper.verify_file(str(TAX / (name + ".case.oath"))))
    assert json.loads(oathkeeper.verify(json.dumps(b), "json")) == report
    save(tax_assets / (name + ".bundle.json"), b)
    tax_cases.append({"name": name, "bundle": b, "report": report})
save(ROOT / "src/data/tax.json", {
    "module": tax_compilation["module"], "rule_sources": tax_rules,
    "sources": json.loads((TAX / "sources.lock.json").read_text())["sources"],
    "review": json.loads((TAX / "review-packet.json").read_text()),
    "cases": tax_cases,
    "fixtures": [{"name": n, "input": json.loads((TAX / "scenarios" / (n + ".json")).read_text()), "baseline": json.loads(oathkeeper.assess_backdoor_roth((TAX / "scenarios" / (n + ".json")).read_text()))} for n in ["synthetic-clean", "synthetic-pro-rata"]],
})
for name,path in {"tax.rs":"crates/oathkeeper-core/src/tax.rs", "arithmetic.rs":"crates/oathkeeper-core/src/arithmetic.rs", "source-tax.oath":"lawlib/us/tax/backdoor-roth-2025.oath", "source-shank.oath":"lawlib/us/tax/shank-basis.oath"}.items():
    shutil.copyfile(CORE/path, ROOT/"src/reference"/name)
    shutil.copyfile(CORE/path, ASSETS/name)
shutil.copyfile(TAX/"README.md", ROOT/"src/docs/TAX.md")
shutil.copyfile(TAX/"README.md", ASSETS/"TAX.md")
package_lock = json.loads((CORE / "lawlib.lock.json").read_text())
save(ROOT / "src/data/lawlib-package.json", {k: package_lock[k] for k in ["repository", "revision", "package_version"]})
shutil.copyfile(CORE / "lawlib.lock.json", ASSETS / "lawlib.lock.json")
shutil.copyfile(CORE / "lawlib/community/current.json", ROOT / "src/data/community.json")
shutil.copyfile(CORE / "lawlib/community/current.json", ASSETS / "community.json")
with zipfile.ZipFile(ASSETS / "oathkeeper-0.5-source.zip", "w", zipfile.ZIP_DEFLATED) as archive:
    source_files = subprocess.check_output(
        ["git", "-C", str(CORE), "ls-files", "-z", "--cached", "--others", "--exclude-standard"]
    ).decode().split("\0")
    for name in sorted(set(source_files) - {""}):
        relative = Path(name)
        path = CORE / relative
        if path.is_file() and not path.is_symlink() and not path.name.startswith(".env") and not path.name.endswith(".key.pem") and not any(part in {".git", ".venv", "target", "__pycache__", "web", "corpus", "artifacts", "proptest-regressions"} for part in relative.parts):
            archive.write(path, Path("oathkeeper") / relative)
print(f"Synced {len(module['rules'])} actual compiled rules and {len(records)} native-verified historical proofs.")
