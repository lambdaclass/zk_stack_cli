cli: 
	cargo +nightly install --path .
	zks autocomplete install

build-cli:
	cargo build --release

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings
