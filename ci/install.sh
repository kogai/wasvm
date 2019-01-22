set -euxo pipefail

main() {
    rustup component add clippy-preview llvm-tools-preview
    case $TARGET in
        thumbv*-none-eabi*)
            rustup target add $TARGET
            ;;
    esac
}

main
