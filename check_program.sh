#!/bin/bash

# Exit on error
set -o errexit
# Exit on error in pipe
set -o pipefail

output=$(domichain-wasm program dump --no-address-labels $1 \
    /dev/stderr 2>&1 >/dev/null | \
    python3 -c '\
    import sys;\
    print(\
        open(sys.argv[1], "rb").read().rstrip(b"\0") ==\
        sys.stdin.buffer.read().rstrip(b"\0")\
    )\
    ' $2)
echo $output