EXAMPLES_SRC := $(shell find ./fixtures -type f -name '*.c')
EXAMPLES_WASM := $(shell find ./dist -type f -name '*.wasm')
TRIPLE := wasm32-unknown-unknown

all: $(EXAMPLES_WASM)

$(EXAMPLES_WASM): $(EXAMPLES_SRC)
	emcc -Oz fixtures/$(shell basename $@ .wasm).c -s WASM=1 -o $(shell basename $@ .wasm).js
	wasm-gc $(shell basename $@ .wasm).wasm -o $@
	wasm2wat $@ -o dist/$(shell basename $@ .wasm).wat
	rm ./$(shell basename $@ .wasm).*

# Prefer to replace Docker container
install:
	packer -S wabt --noconfirm
	cargo install wasm-gc
