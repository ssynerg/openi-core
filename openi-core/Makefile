
## `Makefile`
```make
.PHONY: build fmt check cli node

build:
	cargo build

fmt:
	rustfmt +stable --edition 2021 --emit files .

check:
	cargo check

cli:
	cargo run -p openi-core-cli --

node:
	cargo run -p openi-core-cli -- node
