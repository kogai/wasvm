# EXAMPLES_SRC := $(shell find ./fixtures -type f -name '*.c')
SRC := $(wildcard ./src/*.rc)
TRIPLE := wasm32-unknown-unknown

all: dist/*.wasm

dist/%.wasm: fixtures/%.c
	emcc -O3 -g0 fixtures/$(shell basename $<) -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $< .c).js
	wasm-gc $(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wasm
	wasm2wat dist/$(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wat
	rm ./$(shell basename $< .c).*

report.txt: $(SRC) Makefile
	perf stat -o report.txt cargo run --release fib 30

.PHONY: run
run:
	cargo run --bin main

# Prefer to replace Docker container
install:
	packer -S wabt --noconfirm
	cargo install wasm-gc
