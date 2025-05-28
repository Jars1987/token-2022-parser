.PHONY: all debug release

# Default target: build both debug and release
all: debug release

# Build CLI in debug mode with Rust 1.83
debug:
	rustup run 1.83.0 cargo build --package cli

# Build CLI in release mode with Rust 1.83
release:
	rustup run 1.83.0 cargo build --release --package cli