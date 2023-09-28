deploy:
	~/domichain_backporting/target/release/domichain airdrop 1000 5EXaiabrJmX2TuQ3wTuJJrTiRBZ3Kw93wLoBAduuQo2G
	~/domichain_backporting/target/release/domichain-wasm program deploy program/spl_token_swap.wasm

test:
	cd js && npm run test

deploy_and_test:
	make deploy
	make test

build_domichain_test_validator:
	RUSTFLAGS=-Awarnings cargo build --release --bin domichain-test-validator --manifest-path ~/domichain_backporting/Cargo.toml

build_token_2022:
	~/cargo-wasi/target/debug/cargo-wasi -- build --release --manifest-path ~/domichain-program-library_from_master/token/program-2022/Cargo.toml
	cp ~/domichain-program-library_from_master/target/wasm32-wasi/release/spl_token_2022.wasi.wasm ~/domichain-program-library_from_master/token/program-2022/spl_token_2022.wasm
	cp ~/domichain-program-library_from_master/token/program-2022/spl_token_2022.wasm ~/domichain_backporting/program-test/src/programs/spl_token_2022-0.7.0.wasm

build_token:
	~/cargo-wasi/target/debug/cargo-wasi -- build --release --manifest-path ~/domichain-program-library_from_master/token/program/Cargo.toml
	cp ~/domichain-program-library_from_master/target/wasm32-wasi/release/spl_token.wasi.wasm ~/domichain-program-library_from_master/token/program/spl_token.wasm
	cp ~/domichain-program-library_from_master/token/program/spl_token.wasm ~/domichain_backporting/program-test/src/programs/spl_token-4.0.0.wasm

build_token_swap:
	~/cargo-wasi/target/debug/cargo-wasi -- build --release --manifest-path ~/domichain-program-library_from_master/token-swap/program/Cargo.toml
	cp ~/domichain-program-library_from_master/target/wasm32-wasi/release/spl_token_swap.wasi.wasm ~/domichain-program-library_from_master/token-swap/program/spl_token_swap.wasm

run_test_validator:
	# make build_token
	# make build_token_2022
	make build_domichain_test_validator
	make build_token_swap
	RUST_LOG=OFF ~/domichain_backporting/target/release/domichain-test-validator --wasm-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ~/domichain-program-library_from_master/token-swap/program/spl_token_swap.wasm --reset --log

run_npm_test:
	cd ~/domichain-program-library_from_master/token-swap/js && npm run test

quicktest:
	~/wait_stdout/target/release/wait_stdout 'make run_test_validator' 'Waiting for fees to stabilize' 'make run_npm_test'

run_test_validator_solana:
	RUST_LOG=OFF ~/solana/target/release/solana-test-validator --bpf-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ~/domichain-program-library_from_master/target/debug/deps/libspl_token_swap.so --reset --log

run_test_validator_solana_bin:
	RUST_LOG=OFF solana-test-validator --bpf-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ~/domichain-program-library_from_master/target/debug/deps/libspl_token_swap.so --reset --log


# RUST_LOG=OFF ~/domichain_backporting/target/release/domichain-test-validator --wasm-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ~/domichain-program-library_from_master/token-swap/program/spl_token_swap.wasm --reset --log

# npx start-server-and-test 'RUST_LOG=OFF ~/solana/target/release/solana-test-validator --bpf-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ../../target/deploy/spl_token_swap.so --reset --log' http://127.0.0.1:8899/health test
# npx start-server-and-test 'RUST_LOG=OFF ~/domichain_backporting/target/release/domichain-test-validator --wasm-program SwapsVeCiPHMUAtzQWZw7RjsKjgCjhwU55QGu4U1Szw ~/domichain-program-library_from_master/token-swap/program/spl_token_swap.wasm --reset --log' http://127.0.0.1:8899/health test