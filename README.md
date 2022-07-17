# cw-optimizoor

![alt text](.img/wojak.png)

A blazingly fast alternative to [CosmWasm/rust-optimizer] for compiling & optimizing CW smart contracts.


<!---![Build Status](https://github.com/mandrean/cw-optimizoor/workflows/CI/badge.svg?branch=master)-->
[![Latest version](https://img.shields.io/crates/v/cw-optimizoor.svg)](https://crates.io/crates/cw-optimizoor)
[![Documentation](https://docs.rs/cw-optimizoor/badge.svg)](https://docs.rs/cw-optimizoor)
![License](https://img.shields.io/crates/l/cw-optimizoor.svg)

### Features:

- **Fast** - especially in workspaces with many contracts
- Uses same optimizations as `rust-optimizer` by default
- No dependency on Docker
- Supports both single contracts and workspaces/monorepos
- Written as a [cargo subcommand]
- Cross-platform, cross-arch

### Installation

```sh
$ cargo install cw-optimizoor
```

### Usage

```sh
$ cargo cw-optimizoor -h
cargo-cw-optimizoor 0.1.0

USAGE:
    cargo cw-optimizoor [MANIFEST_PATH]

ARGS:
    <MANIFEST_PATH>    Path to the Cargo.toml
```

### Example
```sh
$ cargo cw-optimizoor monorepo/Cargo.toml

🧐️  Compiling monorepo/Cargo.toml
    Finished release [optimized] target(s) in 0.05s
🥸  Ahh I'm optimiziing
    ...monorepo/target/wasm32-unknown-unknown/release/contract1.wasm
    ...monorepo/target/wasm32-unknown-unknown/release/contract2.wasm
    ...monorepo/target/wasm32-unknown-unknown/release/contract3.wasm
    ...monorepo/target/wasm32-unknown-unknown/release/contract4.wasm
    ...monorepo/target/wasm32-unknown-unknown/release/contract5.wasm
🫡  Done. Saved optimized artifacts to ...monorepo/artifacts
```

[CosmWasm/rust-optimizer]: https://github.com/CosmWasm/rust-optimizer
[CosmWasm]: https://cosmwasm.com
[cargo subcommand]: https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands
