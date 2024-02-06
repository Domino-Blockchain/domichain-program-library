#!/bin/bash

# Exit on error
set -o errexit
# Exit on error in pipe
set -o pipefail

cli_program() {
  declare programid="$1"
  if [[ -z $programid ]]; then
    printf "./target_0/release/spl-token"
  else
    printf "./target_0/release/spl-token --program-id %s" "$programid"
  fi
}

cli=$(cli_program $1)

echo -e "Mint with token program $1\n"

echo -e "Creating token\n";
OUTPUT=$($cli create-token --output json)
TOKEN=$(jq -r .commandOutput.address <<< "$OUTPUT")
echo -e "Created token: $TOKEN\n"

echo -e "Creating account\n"
$cli create-account $TOKEN || exit 1

echo -e "Minting token\n"
$cli mint $TOKEN 100 || exit 1

echo -e "Disabling mint\n"
$cli authorize $TOKEN mint --disable || exit 1

echo -e "Done\n"