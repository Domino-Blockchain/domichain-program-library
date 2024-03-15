scp domi@localhost:~/domichain-program-library/spl*.wasm .
export URL="http://108.48.39.243:8899/"
./domichain-wasm program deploy --url $URL ./spl_token-4.0.0.wasm --program-id ./spl_token-4.0.0-keypair.json
./domichain-wasm program deploy --url $URL ./spl_token-2022-0.6.1.wasm --program-id ./spl_token-2022-0.6.1-keypair.json
./domichain-wasm program deploy --url $URL ./spl_token-btci-4.0.0.wasm --program-id ./spl_token-btci-4.0.0-keypair.json
./domichain-wasm program deploy --url $URL ./spl_associated-token-account-1.0.5.wasm --program-id ./spl_associated-token-account-1.0.5-keypair.json
./domichain-wasm program deploy --url $URL ./spl_token-swap-3.0.0.wasm --program-id ./spl_token-swap-3.0.0-keypair.json