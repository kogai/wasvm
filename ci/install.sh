set -euxo pipefail

main() {
    rustup component add clippy-preview llvm-tools-preview
    cargo install --force wasm-pack
    make discovery/src/discovery_wasm_bg.wasm
    case $TARGET in
        thumbv*-none-eabi*)
            rustup target add $TARGET
            ;;
    esac
}

main
