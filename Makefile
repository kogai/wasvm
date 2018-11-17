SRC := $(wildcard ./src/*.rs)
TRIPLE := wasm32-unknown-unknown

all: dist/*.wasm

dist/%.wasm: fixtures/%.c
	emcc -O3 -g0 fixtures/$(shell basename $<) -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $< .c).js
	wasm-gc $(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wasm
	wasm2wat dist/$(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wat
	rm ./$(shell basename $< .c).*

target/release/main: $(SRC)
	cargo build --release

report.txt: target/release/main Makefile
	perf stat -o report.txt ./target/release/main fib 35

report.node.txt: Makefile
	perf stat -o report.txt node run-wasm fib 35

.PHONY: run
run:
	cargo run --bin main

# Prefer to replace Docker container
install:
	packer -S wabt --noconfirm
	cargo install wasm-gc
