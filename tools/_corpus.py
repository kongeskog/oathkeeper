"""Load tooling from the revision-pinned source-only corpus."""
import sys
sys.dont_write_bytecode = True
from lawlib_package import ROOT, DEST as CORPUS, verify
verify()  # Check the lock before importing code from the external package.
sys.path.insert(0, str(CORPUS / 'tools'))
