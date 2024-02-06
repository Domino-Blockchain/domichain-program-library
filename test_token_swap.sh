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

TOKEN_PROGRAM="7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s"


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
cargo wasi build --release --manifest-path token-swap/program/Cargo.toml --target-dir target_5 & # Token swap
((n+=1))
cd -

for i in $(seq 1 $n);
    do wait -n || { pkill -P $$; sleep 0.5; echo FAILURE; exit 1; }
done
n="0"
wait # Building


echo "Copying"

cd ./domichain-program-library
# 7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s | was TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-4.0.0-keypair.json
cp ./target_1/wasm32-wasi/release/spl_token.wasm ./spl_token-4.0.0.wasm &
# BvVePGKKwuGb6QVJHG6LvCrULB7QBgjocqnYxYHUkNEd | was BvVePGKKwuGb6QVJHG6LvCrULB7QBgjocqnYxYHUkNEd
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-2022-0.6.1-keypair.json
cp ./target_2/wasm32-wasi/release/spl_token_2022.wasm ./spl_token-2022-0.6.1.wasm &
# 5a1Cq6HtzDGzX45xRY7MboGFvdPoLKnUPG7drPTqYn7L
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-btci-4.0.0-keypair.json
cp ./target_3/wasm32-wasi/release/spl_token_btci.wasm ./spl_token-btci-4.0.0.wasm &
# Dt8fRCpjeV6JDemhPmtcTKijgKdPxXHn9Wo9cXY5agtG
# domichain-keygen new --no-bip39-passphrase -o ./spl_associated-token-account-1.0.5-keypair.json
cp ./target_4/wasm32-wasi/release/spl_associated_token_account.wasm ./spl_associated-token-account-1.0.5.wasm &
# 4ZH2DZxr5irLBnfEYtZBVKLWeWnu1uAv9NJkWJyhJvHv
# domichain-keygen new --no-bip39-passphrase -o ./spl_token-swap-3.0.0-keypair.json
cp ./target_5/wasm32-wasi/release/spl_token_swap.wasm ./spl_token-swap-3.0.0.wasm &
cd -

wait # Copying


echo "wasm-strip"

cd ./domichain-program-library/
for i in ./spl_*.wasm;
    do echo $i ; wasmtime --dir=. ~/wabt-1.0.34-wasi/wabt-1.0.34/bin/wasm-strip $i ;
done
cd -

# wasm-strip


# echo "Deploying"
# 
# 7t5SuBhmxxKuQyjwTnmPpFpqJurCDM4dvM14nUGiza4s
# cd ./domichain-program-library
# domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-4.0.0.wasm \
#     --program-id ./spl_token-4.0.0-keypair.json &
# domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-2022-0.6.1.wasm \
#     --program-id ./spl_token-2022-0.6.1-keypair.json &
# domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-btci-4.0.0.wasm \
#     --program-id ./spl_token-btci-4.0.0-keypair.json &
# domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_associated-token-account-1.0.5.wasm \
#     --program-id ./spl_associated-token-account-1.0.5-keypair.json &
# domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-swap-3.0.0.wasm \
#     --program-id ./spl_token-swap-3.0.0-keypair.json &
# cd -

# wait # Deploying


echo "Testing token-swap with token program $TOKEN_PROGRAM"
cd ./domichain-program-library/token-swap/js
# curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash - &&\
# sudo apt-get install -y nodejs
# npm i
npm run test && echo -e "${GREEN}TEST PASSED${NC}" || echo -e "${RED}TEST FAILED${NC}"
cd -

echo DONE