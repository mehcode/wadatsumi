alias fmt := format

# Compile the project.
build:
    @cargo build

# Run the project.
run *args:
    @cargo run -- {{ args }}

# Format source files and ensure license headers.
format:
    @cargo fmt --all
    @hawkeye format --fail-if-updated false

# Check source files
check:
    @cargo check --all --tests

# Lint source files and check for license headers.
lint:
    @cargo clippy --all --tests
    @hawkeye check
    @cargo fmt --check

# Run all tests.
test:
    @cargo test --all

# Run all benchmarks.
bench:
    @cargo bench --all
