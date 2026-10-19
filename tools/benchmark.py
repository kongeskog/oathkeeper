"""Compatibility entry point; canonical tooling lives in pinned lawlib."""
from _corpus import ROOT
from lawlib_tools.benchmark import *

if __name__ == "__main__":
    import argparse,json
    from pathlib import Path
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path)
    parser.add_argument('--timings',action='store_true')
    args=parser.parse_args();result=evaluate(timings=args.timings)
    text=json.dumps(result,indent=2)+'\n'
    if args.output:
        args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(text)
    else:print(text,end='')
    raise SystemExit(0 if result['passed']==result['total'] else 1)
