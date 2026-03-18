set fallback := true

default:
    @just --list

setup:
    cargo fetch

build:
    cargo build --release

test:
    cargo test --workspace

bench: build
    @echo "Phase 1 Validation: Storage Performance"
    time ./target/release/vigil-cli bench --count 100000

sample: build
    @echo "Phase 1 Validation: Chain Integrity"
    rm -rf ./vigil_data
    ./target/release/vigil-cli write -s ontario-line1-temp -v 24.5
    ./target/release/vigil-cli write -s ontario-line1-temp -v 25.0
    ./target/release/vigil-cli verify -s ontario-line1-temp

export-test: build
    @echo "Phase 2 Validation: Air-gap Sync"
    rm -rf ./vigil_data ./backup.car
    ./target/release/vigil-cli write -s test -v 1.0
    ./target/release/vigil-cli write -s test -v 2.0
    ./target/release/vigil-cli export -o backup.car
    @ls -lh backup.car

seed-demo:
    cargo run -p vigil-cli -- seed-demo

detect:
    cargo run -p vigil-cli -- detect

daemon: build
    @echo "Vigil operational dashboard"
    ./target/release/vigil-cli daemon --port 8080

clean:
    cargo clean
    rm -rf ./vigil_data ./backup.car
