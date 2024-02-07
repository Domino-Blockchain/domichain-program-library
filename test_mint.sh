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

domichain transfer ~/.config/domichain/id.json 20 --keypair ~/domichain/config/bootstrap-validator/identity.json --allow-unfunded-recipient

TOKEN_PROGRAM="7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s"

echo "Testing mint of deployed token program $TOKEN_PROGRAM"
cd ~/domichain-program-library
./mint_token.sh "$TOKEN_PROGRAM" && echo -e "${GREEN}TEST PASSED${NC}" || echo -e "${RED}TEST FAILED${NC}"
cd -

echo DONE


# $ domichain balance
# 11 DOMI
# 9.56813912 DOMI

# domichain balance
# 6.04950832 DOMI

# domichain balance
# 96.99358312 DOMI

# domichain balance
# 95.32719832 DOMI

# domichain balance
# 94.77173672 DOMI

# domichain balance
# 94.21627512 DOMI

# domichain balance
# 193.65577424 DOMI

# domichain transfer --keypair config/bootstrap-validator/identity.json ~/.config/domichain/id.json 100


# cargo wasi --version
# cargo-wasi 0.1.28 (e811d4889b 2023-06-12)


# sudo apt update && sudo apt install wabt
# wasm-strip --version
# 1.0.13

# export PATH=~/wabt-1.0.34/bin/:$PATH
# sudo apt-get update
# sudo apt-get install gcc-4.9 
# sudo apt-get upgrade libstdc++6

# sudo add-apt-repository -y ppa:ubuntu-toolchain-r/test
# sudo apt install -y g++-11
# strings /usr/lib/x86_64-linux-gnu/libstdc++.so.6 | grep GLIBCXX

# curl https://wasmtime.dev/install.sh -sSf | bash
# wasmtime ~/wabt-1.0.34-wasi/wabt-1.0.34/bin/wasm-strip --help
