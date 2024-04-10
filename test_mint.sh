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

TOKEN_PROGRAM="TokenAAGbeQq5tGW2r5RoR3oauzN2EkNFiHNPw9q34s"
BTCI_TOKEN_PROGRAM="BTCi9FUjBVY3BSaqjzfhEPKVExuvarj8Gtfn4rJ5soLC"

echo "Testing mint of deployed token program $TOKEN_PROGRAM"
cd ~/domichain-program-library
./mint_token.sh "$TOKEN_PROGRAM" && echo -e "${GREEN}TEST PASSED${NC}" || echo -e "${RED}TEST FAILED${NC}"
cd -

echo "Testing mint of deployed token program $BTCI_TOKEN_PROGRAM"
cd ~/domichain-program-library
./mint_token.sh "$BTCI_TOKEN_PROGRAM" && echo -e "${GREEN}TEST PASSED${NC}" || echo -e "${RED}TEST FAILED${NC}"
cd -

echo DONE

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
# cd ~
# wget https://github.com/WebAssembly/wabt/releases/download/1.0.34/wabt-1.0.34-wasi.tar.gz
# tar -xf wabt-1.0.34-wasi.tar.gz
# wasmtime ~/wabt-1.0.34/bin/wasm-strip --help

