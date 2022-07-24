RUST_SRC=$(wildcard src/*.rs) build.rs
RUST_META=Cargo.toml Cargo.lock
PROJECT=kanso

default: test
all: test run
doc: $(RUST_SRC) $(RUST_META)
	cargo doc
	open target/doc/$(PROJECT)/index.html

test: $(RUST_SRC) $(RUST_META)
	cargo test
run: $(RUST_SRC) $(RUST_META)
	cargo run
clean:
	rm -rf target

.PHONY: test run all clean default doc
