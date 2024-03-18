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

cd ~ # Home dir

export PATH=~/domichain/target/release/:$PATH

TOKEN_PROGRAM="TokenAAGbeQq5tGW2r5RoR3oauzN2EkNFiHNPw9q34s"


echo "Building"
n="0"

# We build each program into separate target folders to prevent cache rebuilding each time.
# Speeds up recompilation.

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
cargo wasi build --release --manifest-path token-swap/program/Cargo.toml --target-dir target_5 & # Token swap
((n+=1))
cd -

cd ./ddex/dex
cargo wasi build --release & # Serum DEX
((n+=1))
cd -

for i in $(seq 1 $n);
    do wait -n || { pkill -P $$; sleep 0.5; echo FAILURE; exit 1; }
done
n="0"
wait # Building


echo "Copying"

cd ./domichain-program-library
cp ./target_1/wasm32-wasi/release/spl_token.wasm ./spl_token-4.0.0.wasm &
cp ./target_2/wasm32-wasi/release/spl_token_2022.wasm ./spl_token-2022-0.6.1.wasm &
cp ./target_3/wasm32-wasi/release/spl_token_btci.wasm ./spl_token-btci-4.0.0.wasm &
cp ./target_4/wasm32-wasi/release/spl_associated_token_account.wasm ./spl_associated-token-account-1.0.5.wasm &
cp ./target_5/wasm32-wasi/release/spl_token_swap.wasm ./spl_token-swap-3.0.0.wasm &
cd -

cp ./ddex/dex/target/wasm32-wasi/release/serum_dex.wasm ./domichain-program-library/serum_dex.wasm &

wait # Copying


echo "wasm-strip"

cd ./domichain-program-library/
for i in ./*.wasm;
    do echo $i ; wasmtime --dir=. ~/wabt-1.0.34/bin/wasm-strip $i ;
done
cd -

# wasm-strip


echo DONE
