SRC := $(wildcard ./src/*.rs)
TRIPLE := wasm32-unknown-unknown
CSRCS=$(wildcard ./fixtures/*.c)
WASTS=$(wildcard ./testsuite/*.wast)
C_WASMS=$(CSRCS:.c=.wasm)
WASMS=$(WASTS:.wast=.wasm)
TEST_CASES=$(WASTS:.wast=.json)

all: $(C_WASMS) $(TEST_CASES)

$(C_WASMS): $(CSRCS)
	emcc -O3 -g0 fixtures/$(shell basename $<) -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $< .c).js
	wasm-gc $(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wasm
	wasm2wat dist/$(shell basename $< .c).wasm -o dist/$(shell basename $< .c).wat
	rm ./$(shell basename $< .c).*

$(TEST_CASES): $(WASTS)
	wast2json $< -o dist/$(shell basename $@)

target/release/main: $(SRC)
	cargo build --release

.PHONY: report.txt
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
