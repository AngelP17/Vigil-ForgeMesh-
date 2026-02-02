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
    time ./target/release/forgemesh-cli bench --count 100000

sample: build
    @echo "Phase 1 Validation: Chain Integrity"
    rm -rf ./forge_data
    ./target/release/forgemesh-cli write -s ontario-line1-temp -v 24.5
    ./target/release/forgemesh-cli write -s ontario-line1-temp -v 25.0
    ./target/release/forgemesh-cli verify -s ontario-line1-temp

export-test: build
    @echo "Phase 2 Validation: Air-gap Sync"
    rm -rf ./forge_data ./backup.car
    ./target/release/forgemesh-cli write -s test -v 1.0
    ./target/release/forgemesh-cli write -s test -v 2.0
    ./target/release/forgemesh-cli export -o backup.car
    @ls -lh backup.car

daemon: build
    @echo "Phase 3 Validation: Distributed Mesh UI"
    ./target/release/forgemesh-cli daemon --port 8080

clean:
    cargo clean
    rm -rf ./forge_data ./backup.car
