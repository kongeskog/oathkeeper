"""Validate authored inputs, exact evidence bytes, and freshly generated proofs."""
import argparse
import json
from pathlib import Path
from . import audit_lawlib, audit_tax, sources
from .build import build
from .runtime import ROOT, check_layout

def check_splits(data):
    seen={}
    for split in ('train','development','held_out'):
        for entry in data.get(split,[]):
            for kind in ('matter','source_family','amendment_lineage'):
                group=entry[kind]
                if not isinstance(group,str) or not group.strip():raise ValueError('evaluation grouping must be explicit')
                key=(kind,group)
                if key in seen and seen[key]!=split:raise ValueError('evaluation leakage across splits: '+kind+' '+group)
                seen[key]=split


def check(root=ROOT, output=None, *, fetch=False):
    check_layout(root)
    evidence=sources.catalog(root)
    for source in evidence:
        (sources.fetch if fetch else sources.require)(source)
    report=build(root,output)
    seattle=audit_lawlib.audit(root/'us/wa/seattle')
    tax=audit_tax.audit(base=root/'us/tax')
    check_splits(json.loads((root/'evaluation-groups.json').read_text()))
    return {'sources':len(evidence),'historical_proof_paths':len(seattle['cases_verified'])+tax['historical_proof_paths'],
            'artifacts_generated':len(report['files']),'evidence_bytes':'verified',
            'held_out_legal_accuracy':'not_measured','status':'engineering_checks_passed'}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT)
    parser.add_argument('--output',type=Path)
    parser.add_argument('--fetch',action='store_true',help='explicitly fetch missing evidence before checking')
    args=parser.parse_args()
    print(json.dumps(check(args.root,args.output,fetch=args.fetch),indent=2))

if __name__=='__main__':main()
