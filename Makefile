SRC := $(wildcard ./src/*.rs)
TRIPLE := wasm32-unknown-unknown
CSRCS=$(wildcard ./fixtures/*.c)
WASTS=$(wildcard ./testsuite/*.wast)
C_WASMS=$(CSRCS:.c=.wasm)
WASMS=$(WASTS:.wast=.wasm)
TEST_CASES=$(WASTS:.wast=.json)

all: $(C_WASMS) $(TEST_CASES)

dist: $(C_WASMS)

$(C_WASMS): $(CSRCS)
	emcc -O3 -g0 fixtures/$(shell basename $@ .wasm).c -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $@ .wasm).js
	wasm-gc $(shell basename $@) -o dist/$(shell basename $@)
	wasm2wat dist/$(shell basename $@) -o dist/$(shell basename $@ .wasm).wat
	rm ./$(shell basename $@ .wasm).*

new_dist: $(TEST_CASES)

$(TEST_CASES): $(WASTS)
	wast2json testsuite/i32.wast -o dist/i32.json
	wasm2wat dist/i32.0.wasm -o dist/i32.wat

wat-dist/%.wasm: wat/%.wat
	wat2wasm $< -o wat-dist/$(shell basename $< .wat).wasm

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
