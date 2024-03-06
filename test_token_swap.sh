#!/bin/bash
#
# please run test_compile.sh and test_deploy.sh first before execute this test
#
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Exit on error
set -o errexit
# Exit on error in pipe
set -o pipefail

# Kill child processes on interrupt
trap 'pkill -P $$; exit 1' SIGINT SIGTERM

cd .. # Home dir

export PATH=~/domichain/target/release/:$PATH

domichain transfer ~/domichain/config/faucet.json 100 --keypair ~/domichain/config/bootstrap-validator/identity.json --allow-unfunded-recipient
TOKEN_PROGRAM="TokenAAGbeQq5tGW2r5RoR3oauzN2EkNFiHNPw9q34s"

echo "Testing token-swap with token program $TOKEN_PROGRAM"
cd ~/domichain-program-library/token-swap/js

# curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - &&\
# sudo apt-get install -y nodejs
# npm i
npm run test && echo -e "${GREEN}TEST PASSED${NC}" || echo -e "${RED}TEST FAILED${NC}"
cd -

echo DONE