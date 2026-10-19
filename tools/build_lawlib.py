"""Generate lawlib products under .build/lawlib without changing source."""
import argparse
import json
from pathlib import Path
from _corpus import ROOT, CORPUS
from lawlib_tools.build import build, archive

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'.build/lawlib')
    parser.add_argument('--check',action='store_true')
    parser.add_argument('--archive',type=Path)
    args=parser.parse_args()
    result=build(CORPUS,args.output,check=args.check)
    if args.archive:archive(args.output,args.archive)
    print(json.dumps({'status':'reproduced' if args.check else 'built','output':str(args.output),'artifacts':len(result['files'])}))

if __name__=='__main__':main()
