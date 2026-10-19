"""Compatibility entry point; canonical tooling lives in pinned lawlib."""
from _corpus import ROOT as SOFTWARE_ROOT
from lawlib_tools.audit_tax import *

if __name__ == "__main__":
    import argparse,json
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--write",action="store_true")
    print(json.dumps(audit(parser.parse_args().write,output=SOFTWARE_ROOT/".build/lawlib/us/tax")))
