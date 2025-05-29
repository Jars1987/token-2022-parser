# Token-2022 Parser

A Rust-based CLI tool for analyzing
[SPL Token-2022](https://github.com/solana-program/token-2022) mint accounts on
Solana.

It fetches all Token-2022 mint accounts, optionally derives associated metadata
PDAs, and identifies which mints have extensions enabled using TLV parsing.

---

## Features

- Fetches all Token-2022 mints from Solana.
- Derives Metadata PDAs using `mpl-token-metadata`.
- Detects and prints all Token-2022 extensions (e.g. `TransferHook`,
  `MetadataPointer`, `MintCloseAuthority`, etc.).
- Support for custom RPC endpoints via `--rpc-url`.

---

## Build Instructions

This project uses **Rust 1.83**. Make sure it’s installed:

```bash
rustup install 1.83.0
rustup override set 1.83.0
```

Then, build the project using the provided Makefile:

```bash
make debug      # Builds the CLI in debug mode
make release    # Builds the CLI in release mode
```

## Usage

Once built, you can run the CLI using Cargo:

```bash
cargo run -- get-tokens-with-metadata-account
cargo run -- get-tokens-with-extensions
```

Or if you've built a release binary:

```bash
./spl-token-2022-cli get-tokens-with-metadata-account
./spl-token-2022-cli get-tokens-with-extensions

```

You can also pass a custom RPC URL:

```bash
cargo run -- get-tokens-with-extensions --rpc-url https://api.mainnet-beta.solana.com

```

## Available Commands

get-tokens-with-metadata-account: Fetch all Token-2022 mints and print those
with valid metadata accounts.

get-tokens-with-extensions: Fetch all Token-2022 mints and print those with one
or more TLV-based token extensions.

To see full help:

```
cargo run -- --help
cargo run -- get-tokens-with-extensions --help
```

## License

MIT
