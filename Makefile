SRC := $(wildcard *.rs)
TRIPLE := wasm32-unknown-unknown
CSRCS=$(wildcard ./fixtures/*.c)
WASTS=$(filter-out "./testsuite/binary.json", $(wildcard ./testsuite/*.wast))
BENCH_DIR := life/bench/cases
C_WASMS=$(CSRCS:.c=.wasm)
WASMS=$(WASTS:.wast=.wasm)
TARGET := thumbv7em-none-eabihf
ARM_GDB := arm-none-eabi-gdb
DISCOVERY := discovery

all: $(C_WASMS)
dist: $(C_WASMS)

discovery/debug: discovery/target/$(TARGET)/debug/$(DISCOVERY)
discovery/release: discovery/target/$(TARGET)/release/$(DISCOVERY)
discovery: discovery/debug discovery/release

$(C_WASMS): $(CSRCS)
	emcc -O3 -g0 fixtures/$(shell basename $@ .wasm).c -s "EXPORTED_FUNCTIONS=['_subject', '_f', '_g']" -s WASM=1 -o $(shell basename $@ .wasm).js
	wasm-gc $(shell basename $@) -o dist/$(shell basename $@)
	wasm2wat dist/$(shell basename $@) -o dist/$(shell basename $@ .wasm).wat
	rm ./$(shell basename $@ .wasm).*

discovery/target/$(TARGET)/debug/$(DISCOVERY): $(SRC)
	cd discovery && \
	cargo build

discovery/target/$(TARGET)/release/$(DISCOVERY): $(SRC)
	cd discovery && \
	cargo build --release

.PHONY: openocd gdb/debug gdb/release
openocd:
	openocd -f discovery/openocd.cfg

gdb/debug:
	$(ARM_GDB) -x discovery/openocd.gdb discovery/target/$(TARGET)/debug/$(DISCOVERY)

gdb/release:
	$(ARM_GDB) -x discovery/openocd.gdb discovery/target/$(TARGET)/release/$(DISCOVERY)

target/release/main: $(SRC) Makefile
	RUSTFLAGS='-g' cargo build --release

.PHONY: report.txt
report.txt: target/release/main Makefile
	perf stat -o report.txt ./target/release/main dist/fib 35

.PHONY: out.perf
out.perf: target/release/main Makefile
	perf record --call-graph=lbr ./target/release/main dist/fib 35
	# perf record -g -- node run-wasm.js dist/fib subject 35
	perf script > out.perf
	# perf report -g fractal --sort dso,comm
	# perf report -n --stdio

out.svg: out.perf
	./FlameGraph/stackcollapse-perf.pl out.perf > out.perf_folded
	./FlameGraph/flamegraph.pl out.perf_folded > out.svg

report.node.txt: Makefile
	perf stat -o report.node.txt node run-wasm dist/fib _subject 35

.PHONY: run
run:
	cargo run --bin main

.PHONY: benches
benches: tmp/fib_recursive.wasm tmp/pollard_rho_128.wasm tmp/snappy_compress.wasm

tmp/fib_recursive.wasm:
	cargo build --release --target=$(TRIPLE) --manifest-path=$(BENCH_DIR)/fib_recursive/Cargo.toml
	mv $(BENCH_DIR)/fib_recursive/target/$(TRIPLE)/release/fib_recursive.wasm tmp/

tmp/pollard_rho_128.wasm:
	cargo build --release --target=$(TRIPLE) --manifest-path=$(BENCH_DIR)/pollard_rho_128/Cargo.toml
	mv $(BENCH_DIR)/pollard_rho_128/target/$(TRIPLE)/release/pollard_rho_128.wasm tmp/

tmp/snappy_compress.wasm:
	cargo build --release --target=$(TRIPLE) --manifest-path=$(BENCH_DIR)/snappy_compress/Cargo.toml
	mv $(BENCH_DIR)/snappy_compress/target/$(TRIPLE)/release/snappy_compress.wasm tmp/

# Prefer to replace Docker container
install:
	packer -S wabt --noconfirm
	cargo install wasm-gc cargo-binutils cargo-bloat
	sudo pacman -S \
		arm-none-eabi-gdb \
		qemu-arch-extra
