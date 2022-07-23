RUST_SRC=$(wildcard src/*.rs)
PROJECT=kanso

default: test
all: test run
doc: $(RUST_SRC) Cargo.toml
	cargo doc
	open target/doc/$(PROJECT)/index.html

test: $(RUST_SRC) Cargo.toml
	cargo test
run: $(RUST_SRC) Cargo.toml
	cargo run
clean:
	rm -rf target

.PHONY: test run all clean default doc
