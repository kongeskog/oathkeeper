"""Regenerate JSON interchange from canonical .oath sources; never edit rules in JSON."""
import argparse
import json
from pathlib import Path
from audit_lawlib import compile_source, DEFAULT


def export(base=DEFAULT):
    module = compile_source(base / "parking.oath")["module"]
    (base / "parking.module.json").write_text(json.dumps(module, indent=2) + "\n")
    count = 0
    for path in sorted((base / "cases").glob("*.case.oath")):
        case = compile_source(path)["case"]
        path.with_suffix(".json").write_text(json.dumps(case, indent=2) + "\n")
        count += 1
    return {"rules": len(module["rules"]), "cases": count}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("base", type=Path, nargs="?", default=DEFAULT)
    print(json.dumps(export(parser.parse_args().base)))
