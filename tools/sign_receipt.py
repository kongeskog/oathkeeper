"""Sign a receipt with an existing Ed25519 PEM key. Requires cryptography.

The private key is read locally and is never printed. Keep it outside repositories.
The Rust binding defines the signing bytes; JSON serialization is not duplicated.
"""
import argparse
import json
from pathlib import Path
import oathkeeper

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('receipt',type=Path)
    parser.add_argument('--private-key',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    from cryptography.hazmat.primitives.serialization import load_pem_private_key
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    key=load_pem_private_key(args.private_key.read_bytes(),password=None)
    if not isinstance(key,Ed25519PrivateKey):
        raise ValueError('an Ed25519 private key is required')
    event=json.loads(args.receipt.read_text())
    event['signature']=''
    body=json.loads(oathkeeper.assess_policy(json.dumps({'operation':'signing_payload','receipt':event})))['utf8']
    event['signature']=key.sign(body.encode()).hex()
    # Create a new file rather than overwrite the key, draft, or an existing receipt.
    with args.output.open('x') as stream:
        json.dump(event,stream,indent=2);stream.write('\n')

if __name__=='__main__':main()
