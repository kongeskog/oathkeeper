"""Refresh application artifacts from the actual native Oathkeeper compiler/kernel."""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

import oathkeeper

ROOT = Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else Path(__file__).resolve().parents[1]
CORE = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT.parent
BUILD = CORE / ".build/lawlib"
GENERATED = ROOT / ".generated"
sys.dont_write_bytecode = True
sys.path.insert(0, str(CORE / 'tools'))
from _corpus import CORPUS
from lawlib_tools.sources import require
from lawlib_tools.runtime import module_paths
SEATTLE_INPUTS = module_paths(CORPUS, 'seattle')
TAX_INPUTS = module_paths(CORPUS, 'ira2025')
BASE = SEATTLE_INPUTS['entrypoint'].parent
if GENERATED.exists():
    shutil.rmtree(GENERATED)
for part in ["data", "docs", "reference", "public/assets"]:
    (GENERATED / part).mkdir(parents=True, exist_ok=True)
shutil.copytree(ROOT / "public", GENERATED / "public", dirs_exist_ok=True)
ASSETS = GENERATED / "public/assets"

def save(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")

compilation = json.loads(oathkeeper.compile_file(str(BASE / "parking.oath")))
module = compilation["module"]
save(GENERATED / "data/module.json", module)
save(ASSETS / "parking.module.json", module)
seattle_exports = {
    name: BUILD / 'us/wa/seattle' / origin for name, origin in {
        'coverage.json': 'coverage.json', 'sources.lock.json': 'sources.lock.json',
        'interpretations.json': 'parking.interpretations.json',
        'oath.lock.json': 'oath.lock.json', 'review-packet.json': 'review-packet.json',
        'baseline.json': 'baseline.json',
    }.items()
}
for name, origin in seattle_exports.items():
    shutil.copyfile(origin, ASSETS / name)
    shutil.copyfile(origin, GENERATED / "data" / name)
records = json.loads((ROOT / "src/catalog/cases.json").read_text())
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
save(GENERATED / "data/cases.json", records)
save(GENERATED / "data/case-sources.json", case_sources)
def documented_lines(path, span):
    lines = path.read_text().splitlines()
    start = span['line'] - 1
    while start > 0 and lines[start - 1].lstrip().startswith('///'):
        start -= 1
    return lines[start:span['end_line']]

rule_sources = {}
for span in compilation["source_map"]:
    if span["kind"] != "rule":
        continue
    path = Path(span["file"])
    rule_sources[span["id"]] = {"file": 'oathkeeper-lawlib/' + path.relative_to(CORPUS).as_posix(), "line": span["line"], "code": "\n".join(documented_lines(path, span))}
save(GENERATED / "data/rule-sources.json", rule_sources)
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
    origin = CORPUS / path.removeprefix('lawlib/') if path.startswith('lawlib/') else CORE / path
    shutil.copyfile(origin, GENERATED / "reference" / name)
    shutil.copyfile(origin, ASSETS / name)
for name, path in {
    "CLI.md":"docs/CLI.md", "PYTHON.md":"docs/PYTHON.md", "ORACLES.md":"docs/ORACLES.md", "SPEC.md":"docs/SPEC-0.1.md", "STORAGE.md":"lawlib/docs/STORAGE.md",
    "SPEC-0.5.md":"docs/SPEC-0.5.md", "SPEC-0.4.md":"docs/SPEC-0.4.md", "RESEARCH.md":"docs/RESEARCH.md", "SOUNDNESS.md":"docs/SOUNDNESS.md",
    "GETTING_STARTED.md":"README.md", "SPEC-0.3.md":"docs/SPEC-0.3.md", "SPEC-0.2.md":"docs/SPEC-0.2.md",
    "ARCHITECTURE.md":"docs/ARCHITECTURE.md", "TRAINING.md":"docs/TRAINING.md", "SEATTLE.md":"lawlib/us/wa/seattle/README.md",
    "SEATTLE_PLAN.md":"docs/SEATTLE_PLAN.md", "ROADMAP.md":"docs/ROADMAP.md",
}.items():
    origin = CORPUS / path.removeprefix('lawlib/') if path.startswith('lawlib/') else CORE / path
    shutil.copyfile(origin, GENERATED / "docs" / name)
    shutil.copyfile(origin, ASSETS / name)
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
save(GENERATED / "data/workbench-cases.json", workbench)
for source in json.loads((BUILD / 'us/wa/seattle/sources.lock.json').read_text())["sources"]:
    shutil.copyfile(require(source), ASSETS / Path(source["file"]).name)
# The federal pilot is a separate versioned module, never merged into Seattle's scope.
TAX = TAX_INPUTS['entrypoint'].parent
tax_assets = ASSETS / "tax"
tax_assets.mkdir(exist_ok=True)
for path in TAX.rglob("*"):
    if path.is_file():
        target = tax_assets / path.relative_to(TAX)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
# Public download URLs remain stable; authored metadata and fixtures are now
# resolved through the package manifest, separately from legal source.
for name, origin in {'sources.lock.json': 'sources.lock.json', 'interpretations.json': 'backdoor-roth-2025.interpretations.json'}.items():
    shutil.copyfile(BUILD / 'us/tax' / origin, tax_assets / name)
shutil.copytree(TAX_INPUTS['scenarios'], tax_assets / 'scenarios')
for path in (BUILD / "us/tax").rglob("*"):
    if path.is_file():
        target = tax_assets / path.relative_to(BUILD / "us/tax")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
for source in json.loads((BUILD / 'us/tax/sources.lock.json').read_text())["sources"]:
    target = tax_assets / source["file"]
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(require(source), target)
tax_compilation = json.loads(oathkeeper.compile_file(str(TAX / "backdoor-roth-2025.oath")))
tax_rules = {}
for span in tax_compilation["source_map"]:
    if span["kind"] == "rule":
        path = Path(span["file"])
        tax_rules[span["id"]] = "\n".join(documented_lines(path, span))
tax_cases = []
for name in ["shank-2018", "shank-unavailable-returns"]:
    c = json.loads(oathkeeper.compile_file(str(TAX / (name + ".case.oath"))))
    b = {**c["module"], **c["case"]}
    b["constants"] = c["module"].get("constants", []) + c["case"].get("constants", [])
    report = json.loads(oathkeeper.verify_file(str(TAX / (name + ".case.oath"))))
    assert json.loads(oathkeeper.verify(json.dumps(b), "json")) == report
    save(tax_assets / (name + ".bundle.json"), b)
    tax_cases.append({"name": name, "bundle": b, "report": report})
save(GENERATED / "data/tax.json", {
    "module": tax_compilation["module"], "rule_sources": tax_rules,
    "sources": json.loads((BUILD / 'us/tax/sources.lock.json').read_text())["sources"],
    "review": json.loads((BUILD / "us/tax/review-packet.json").read_text()),
    "cases": tax_cases,
    "fixtures": [{"name": n, "input": json.loads((TAX_INPUTS['scenarios'] / (n + ".json")).read_text()), "baseline": json.loads(oathkeeper.assess_backdoor_roth((TAX_INPUTS['scenarios'] / (n + ".json")).read_text()))} for n in ["synthetic-clean", "synthetic-pro-rata"]],
})
for name,path in {"tax.rs":"crates/oathkeeper-core/src/tax.rs", "arithmetic.rs":"crates/oathkeeper-core/src/arithmetic.rs", "source-tax.oath":"lawlib/us/tax/backdoor-roth-2025.oath", "source-shank.oath":"lawlib/us/tax/shank-basis.oath"}.items():
    origin = CORPUS / path.removeprefix('lawlib/') if path.startswith('lawlib/') else CORE / path
    shutil.copyfile(origin, GENERATED/"reference"/name)
    shutil.copyfile(origin, ASSETS/name)
shutil.copyfile(TAX/"README.md", GENERATED/"docs/TAX.md")
shutil.copyfile(TAX/"README.md", ASSETS/"TAX.md")
package_lock = json.loads((CORE / "lawlib.lock.json").read_text())
save(GENERATED / "data/lawlib-package.json", {k: package_lock[k] for k in ["repository", "revision", "package_version"]})
shutil.copyfile(CORE / "lawlib.lock.json", ASSETS / "lawlib.lock.json")
shutil.copyfile(BUILD / "community/current.json", GENERATED / "data/community.json")
shutil.copyfile(BUILD / "community/current.json", ASSETS / "community.json")
with zipfile.ZipFile(ASSETS / "oathkeeper-0.5-source.zip", "w", zipfile.ZIP_DEFLATED) as archive:
    source_files = subprocess.check_output(
        ["git", "-C", str(CORE), "ls-files", "-z", "--cached", "--others", "--exclude-standard"]
    ).decode().split("\0")
    for name in sorted(set(source_files) - {""}):
        relative = Path(name)
        path = CORE / relative
        if path.is_file() and not path.is_symlink() and not path.name.startswith(".env") and not path.name.endswith(".key.pem") and not any(part in {".git", ".venv", ".deps", ".build", "target", "__pycache__", "web", "corpus", "artifacts", "proptest-regressions"} for part in relative.parts):
            archive.write(path, Path("oathkeeper") / relative)
print(f"Synced {len(module['rules'])} actual compiled rules and {len(records)} native-verified historical proofs.")
