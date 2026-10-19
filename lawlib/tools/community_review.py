"""Generate an artifact-bound review report under .build/."""
import sys
sys.dont_write_bytecode = True
from lawlib_tools.community_review import main

if __name__ == "__main__":
    main()
