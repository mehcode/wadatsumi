# Compile the project.
build:
    @cargo build

# Run the project.
run *args:
    @cargo run -- {{ args }}

# Format source files and ensure license headers.
format:
    @cargo fmt
    @hawkeye format --fail-if-updated false

# Lint source files and check for license headers.
lint:
    @cargo clippy
    @hawkeye check
    @cargo fmt --check

# Run all tests.
test:
    @cargo test
