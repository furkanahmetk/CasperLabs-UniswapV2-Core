prepare:
	rustup target add wasm32-unknown-unknown

build-contract:
	cargo build --release -p flashswapper --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/flashswapper-token.wasm 2>/dev/null | true

test-only:
	cargo test -p flashswapper-tests

copy-wasm-file-to-test:
	cp target/wasm32-unknown-unknown/release/*.wasm flashswapper-tests/wasm

test: build-contract copy-wasm-file-to-test test-only

clippy:
	cargo clippy --all-targets --all -- -D warnings

check-lint: clippy
	cargo fmt --all -- --check

lint: clippy
	cargo fmt --all

clean:
	cargo clean
	rm -rf flashswapper-tests/wasm/*.wasm
