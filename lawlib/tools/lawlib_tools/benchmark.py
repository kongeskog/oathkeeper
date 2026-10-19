"""Run deterministic proof search before RL; report engineering outcomes, not legal accuracy."""
import argparse
import copy
import json
from pathlib import Path
import time
import oathkeeper

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "us/wa/seattle"


def bundle(path):
    compiled = json.loads(oathkeeper.compile_file(str(path)))
    result = {**compiled["module"], **compiled["case"]}
    result["constants"] = compiled["module"].get("constants", []) + compiled["case"].get("constants", [])
    result["proof"] = []
    return result


def cases(base=BASE):
    BASE = base
    coverage = json.loads((BASE / "coverage.json").read_text())
    for path in coverage["cases"]:
        yield {"id":Path(path).stem,"origin":"historical_reconstruction","group":"long-98824-2","expected":"supported","context":bundle(BASE/path),"judicial_findings_supplied":True,"evaluation_use":"reconstruction, not outcome prediction"}
    for path in coverage["assessment_cases"] + coverage["pending_cases"]:
        context = bundle(BASE/path)
        yield {"id":Path(path).stem,"origin":"assessment_over_real_record","group":"long-98824-2","expected":"neither","context":context,"judicial_findings_supplied":any(f["basis"]["kind"] == "judicial_finding" for f in context["facts"])}
    original = bundle(BASE / "cases/long-parking-evidence-review.case.oath")
    # Clearly labeled synthetic contexts around a real enacted rule. These are
    # boundary tests, never fictional court decisions placed in lawlib.
    for duration, expected in [(259199999,"neither"),(259200000,"neither"),(259200001,"supported"),(261000000,"supported")]:
        context = copy.deepcopy(original)
        context["query"]["purpose"] = "synthetic_test"
        context["query"]["temporal_rationale"] = "Synthetic boundary mutation of the historical parking question; no real-world outcome asserted."
        context["facts"] = [f for f in context["facts"] if f["id"] != "observed-duration"]
        template = copy.deepcopy(context["facts"][0])
        template["basis"].update(kind="scope_assumption",source={"citation":"Synthetic benchmark inputs","uri":"urn:oathkeeper:benchmark","sha256":"1"*64},pinpoint="boundary test",explanation="Synthetic premise; not a finding about the real Long matter.")
        template["evidence_sha256"] = "1"*64
        for i,atom in enumerate(["parking.shorter_posted_limit","parking.otherwise_provided_by_law","parking.smc72_c_excluded","parking.junk_motor_vehicle","parking.abandoned_junk_motor_vehicle"]):
            fact = copy.deepcopy(template);fact["id"] = f"test-negative-{i}";fact["claim"] = {"atom":atom,"negative":True,"arguments":[{"kind":"symbol","value":"long.98824-2"}]};context["facts"].append(fact)
        fact = copy.deepcopy(template);fact["id"]="test-duration";fact["claim"]={"atom":"parking.duration","negative":False,"arguments":[{"kind":"symbol","value":"long.98824-2"},{"kind":"quantity","unit":"ms","value":duration}]};context["facts"].append(fact)
        yield {"id":f"synthetic-duration-{duration}","origin":"synthetic_boundary_test","group":"smc-11.72.440","expected":expected,"context":context,"judicial_findings_supplied":False}


def evaluate(base=BASE, *, timings=False):
    results=[]
    for task in cases(base):
        start=time.perf_counter()
        assessment=json.loads(oathkeeper.analyze(json.dumps(task["context"]),json.dumps({"sensitivity":False})))
        certificates=assessment["supporting"]+assessment["opposing"]
        # Replay every certificate with the ordinary checker, including opposition.
        for argument in certificates:
            replay=copy.deepcopy(task["context"]);replay["query"]["goal"]=argument["report"]["goal"];replay["proof"]=argument["proof"]
            assert json.loads(oathkeeper.verify(json.dumps(replay),"json"))["bundle_sha256"]==argument["report"]["bundle_sha256"]
        ok=assessment["status"]==task["expected"] and assessment["search"]["complete_for_ground_domain"]
        results.append({k:v for k,v in task.items() if k!="context"} | {"actual":assessment["status"],"passed":ok,"certificate_count":len(certificates),"search_complete":assessment["search"]["complete_for_ground_domain"],"elapsed_ms":round((time.perf_counter()-start)*1000,3) if timings else None,"context_sha256":assessment["context_sha256"]})
    return {"format":"oathkeeper-baseline/1","kernel_version":oathkeeper.__version__,"baseline":"deterministic finite-domain proof search; no LLM or RL training","tasks":results,"passed":sum(r["passed"] for r in results),"total":len(results),"independent_authentic_matters":1,"held_out_legal_generalization":"not_evaluated; all authentic records belong to one litigation matter","legal_accuracy":"not_measured; expectations are declared engineering fixtures awaiting legal review","practitioner_comparison":"not_performed"}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output=evaluate()
    text=json.dumps(output,indent=2)+"\n"
    if args.output:
        args.output.write_text(text)
    else:
        print(text,end="")
    raise SystemExit(0 if output["passed"]==output["total"] else 1)
