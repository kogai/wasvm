set -euxo pipefail

main() {

    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo clippy
        cargo test --target $TARGET
    else 
      cd discovery
      cargo check --target $TARGET
    fi
}

main
