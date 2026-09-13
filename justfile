default: build

build:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

install:
	cargo install --path .
