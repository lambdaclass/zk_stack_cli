cli: 
	cargo install --path .

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings