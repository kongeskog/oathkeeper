"""Build app inputs from pinned corpus source, cached evidence and the Rust core."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
from _corpus import ROOT
from lawlib_tools.build import build
from lawlib_tools import sources
from lawlib_package import verify

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--web',type=Path,default=ROOT/'web',help='app source checkout, including a managed Sites checkout')
    parser.add_argument('--fetch-sources',action='store_true',help='explicitly fetch missing evidence for the selected app modules')
    args=parser.parse_args()
    verify()
    pin=args.web/'.openai/build-inputs.json'
    if pin.exists():
        expected=json.loads(pin.read_text())['engine_revision']
        actual=subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD'],text=True).strip()
        if actual!=expected:raise ValueError('app source pins a different engine revision')
    for source in sources.catalog(ROOT/'lawlib',['seattle','ira2025','shank']):
        (sources.fetch if args.fetch_sources else sources.require)(source)
    build(ROOT/'lawlib',ROOT/'.build/lawlib')
    subprocess.run([sys.executable,str(ROOT/'web/scripts/sync-lawlib.py'),str(ROOT),str(args.web)],check=True)
    subprocess.run(['cargo','build','-p','oathkeeper-wasm','--target','wasm32-unknown-unknown','--release','--locked'],cwd=ROOT,check=True)
    subprocess.run(['wasm-bindgen',str(ROOT/'target/wasm32-unknown-unknown/release/oathkeeper_wasm.wasm'),
                    '--target','web','--out-dir',str(args.web/'.generated/public/assets/kernel'),'--out-name','oathkeeper_wasm'],check=True)
    print('App inputs generated under '+str(args.web/'.generated')+'; run npm run build in the app checkout.')

if __name__=='__main__':main()
