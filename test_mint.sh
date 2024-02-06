#!/bin/bash

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

export PATH=./domichain/target/release/:$PATH

TOKEN_PROGRAM=$(domichain-keygen pubkey ./domichain-program-library/spl_token-4.0.0-keypair.json)


echo "Building"
n="0"

cd ./domichain-program-library
cargo build --release --manifest-path token/cli/Cargo.toml --target-dir target_0 & # CLI
((n+=1))
cargo wasi build --release --manifest-path token/program/Cargo.toml --target-dir target_1 & # Token
((n+=1))
cargo wasi build --release --manifest-path token/program-2022/Cargo.toml --target-dir target_2 & # Token2022
((n+=1))
cargo wasi build --release --manifest-path token/program-btci/Cargo.toml --target-dir target_3 & # Token BTCi
((n+=1))
cargo wasi build --release --manifest-path associated-token-account/program/Cargo.toml --target-dir target_4 & # Associated Token
((n+=1))
cd -

for i in $(seq 1 $n);
    do wait -n || { pkill -P $$; sleep 0.5; echo FAILURE; exit 1; }
done
n="0"
wait # Building


echo "Copying"

cd ./domichain-program-library
# 7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-4.0.0-keypair.json
cp ./target_1/wasm32-wasi/release/spl_token.wasm ./spl_token-4.0.0.wasm &
# BvVePGKKwuGb6QVJHG6LvCrULB7QBgjocqnYxYHUkNEd
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-2022-0.6.1-keypair.json
cp ./target_2/wasm32-wasi/release/spl_token_2022.wasm ./spl_token-2022-0.6.1.wasm &
# 5a1Cq6HtzDGzX45xRY7MboGFvdPoLKnUPG7drPTqYn7L
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-btci-4.0.0-keypair.json
cp ./target_3/wasm32-wasi/release/spl_token_btci.wasm ./spl_token-btci-4.0.0.wasm &
# Dt8fRCpjeV6JDemhPmtcTKijgKdPxXHn9Wo9cXY5agtG
# domichain-keygen new --no-bip39-passphrase -o ./spl_associated-token-account-1.0.5-keypair.json
cp ./target_4/wasm32-wasi/release/spl_associated_token_account.wasm ./spl_associated-token-account-1.0.5.wasm &
cd -

wait # Copying


echo "wasm-strip"

cd ./domichain-program-library/
for i in ./spl_*.wasm;
    do echo $i ; wasmtime --dir=. ~/wabt-1.0.34-wasi/wabt-1.0.34/bin/wasm-strip $i ;
done
cd -

# wasm-strip


echo "Deploying"

# 7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s
cd ./domichain-program-library
domichain-wasm program deploy ./spl_token-4.0.0.wasm \
    --program-id ./spl_token-4.0.0-keypair.json &
domichain-wasm program deploy ./spl_token-2022-0.6.1.wasm \
    --program-id ./spl_token-2022-0.6.1-keypair.json &
domichain-wasm program deploy ./spl_token-btci-4.0.0.wasm \
    --program-id ./spl_token-btci-4.0.0-keypair.json &
domichain-wasm program deploy ./spl_associated-token-account-1.0.5.wasm \
    --program-id ./spl_associated-token-account-1.0.5-keypair.json &
cd -

wait # Deploying


echo "Testing mint of deployed token program $TOKEN_PROGRAM"
cd ./domichain-program-library
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
