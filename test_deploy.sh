scp domi@localhost:~/domichain-program-library/spl*.wasm .
./domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-4.0.0.wasm --program-id ./spl_token-4.0.0-keypair.json
./domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-2022-0.6.1.wasm --program-id ./spl_token-2022-0.6.1-keypair.json
./domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-btci-4.0.0.wasm --program-id ./spl_token-btci-4.0.0-keypair.json
./domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_associated-token-account-1.0.5.wasm --program-id ./spl_associated-token-account-1.0.5-keypair.json
./domichain-wasm program deploy --url http://108.48.39.243:8899/ ./spl_token-swap-3.0.0.wasm --program-id ./spl_token-swap-3.0.0-keypair.json