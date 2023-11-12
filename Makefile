cli: 
	cargo install --path .

build-cli:
	cargo build --release

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings
