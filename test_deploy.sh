#!/bin/bash

# Exit on error
set -o errexit

echo "Deploying smart contracts on the Devnet"
export URL="http://108.48.39.242:8899/" # Devnet

n="0"

# TokenAAGbeQq5tGW2r5RoR3oauzN2EkNFiHNPw9q34s
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./spl_token-4.0.0.wasm \
  --program-id ~/keys_spl_tokens/TokenAAGbeQq5tGW2r5RoR3oauzN2EkNFiHNPw9q34s.json &
((n+=1))

# BvVePGKKwuGb6QVJHG6LvCrULB7QBgjocqnYxYHUkNEd
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./spl_token-2022-0.6.1.wasm \
  --program-id ~/keys_spl_tokens/spl_token-2022-0.6.1-keypair.json \
  --keypair ~/keys_spl_tokens/domip_id.json # dserver2 domip identity &
((n+=1))

# BTCi9FUjBVY3BSaqjzfhEPKVExuvarj8Gtfn4rJ5soLC
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./spl_token-btci-4.0.0.wasm \
  --program-id ~/keys_spl_tokens/BTCi9FUjBVY3BSaqjzfhEPKVExuvarj8Gtfn4rJ5soLC.json &
((n+=1))

# Dt8fRCpjeV6JDemhPmtcTKijgKdPxXHn9Wo9cXY5agtG
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./spl_associated-token-account-1.0.5.wasm \
  --program-id ~/keys_spl_tokens/spl_associated-token-account-1.0.5-keypair.json \
  --keypair ~/keys_spl_tokens/domip_id.json # dserver2 domip identity &
((n+=1))

# SwapLyqLfyxTHPYf3A3sUYS9qHo2jCFXyhJP4w2UVUd
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./spl_token-swap-3.0.0.wasm \
  --program-id ~/keys_spl_tokens/SwapLyqLfyxTHPYf3A3sUYS9qHo2jCFXyhJP4w2UVUd.json &
((n+=1))

# DexobvLtDf7UbtNJQgf5SsuExkS1JaftvAMNsnEiAvxL
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./serum_dex.wasm \
  --program-id ~/keys_spl_tokens/DexobvLtDf7UbtNJQgf5SsuExkS1JaftvAMNsnEiAvxL.json &
((n+=1))

# MetaXKaVt8cn9dGYns81au23cqBYUH4DU4WpC8tAbhQ
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./token_metadata.wasm \
  --program-id ~/keys_spl_tokens/MetaXKaVt8cn9dGYns81au23cqBYUH4DU4WpC8tAbhQ.json &
((n+=1))

# 1NSA9E2dwbXfhmvP3VnnjpT8G5R89qnyw7AkXCjhzoB
~/domichain/target/release/domichain-wasm program deploy \
  --url $URL \
  ./mpl_inscription_program.wasm \
  --program-id ~/keys_spl_tokens/1NSA9E2dwbXfhmvP3VnnjpT8G5R89qnyw7AkXCjhzoB.json &
((n+=1))

for i in $(seq 1 $n);
    do wait -n || { pkill -P $$; sleep 0.5; echo FAILURE; exit 1; }
done
n="0"

echo DONE
