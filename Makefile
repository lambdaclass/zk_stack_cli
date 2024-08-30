cli: 
	cargo +nightly install --path .
	zks autocomplete install

build-cli:
	cargo +nightly build --release

fmt:
	cargo +nightly fmt --all

clippy:
	cargo +nightly clippy --all-targets -- -D warnings
