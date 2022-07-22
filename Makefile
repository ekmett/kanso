RUST_SRC=$(wildcard src/*.rs)
default: test
all: test run
test: $(RUST_SRC) Cargo.toml
	cargo test
run: $(RUST_SRC) Cargo.toml
	cargo run
clean:
	rm -rf target

.PHONY: test run all clean default
