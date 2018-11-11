EXAMPLES_SRC := $(shell find ./fixtures -type f -name '*.c')
TRIPLE := wasm32-unknown-unknown

all: dist/*.wasm

dist/%.wasm: fixtures/%.c 
	emcc -Oz fixtures/$(shell basename $@ .wasm).c -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $@ .wasm).js
	wasm-gc $(shell basename $@ .wasm).wasm -o $@
	wasm2wat $@ -o dist/$(shell basename $@ .wasm).wat
	rm ./$(shell basename $@ .wasm).*

.PHONY: run
run:
	cargo run --bin main

# Prefer to replace Docker container
install:
	packer -S wabt --noconfirm
	cargo install wasm-gc
