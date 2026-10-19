"""Validate the source-only corpus and its external evidence."""
import sys
sys.dont_write_bytecode=True
from lawlib_tools.check import check, check_splits, main

if __name__=="__main__":
    main()
