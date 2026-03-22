# cw-optimizoor

![cw-optimizoor mascot](.img/wojak.png)

[![CI](https://github.com/mandrean/cw-optimizoor/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/mandrean/cw-optimizoor/actions/workflows/ci.yml)
[![Release-plz](https://github.com/mandrean/cw-optimizoor/actions/workflows/release-plz.yml/badge.svg?branch=master)](https://github.com/mandrean/cw-optimizoor/actions/workflows/release-plz.yml)
[![Latest version](https://img.shields.io/crates/v/cw-optimizoor.svg)](https://crates.io/crates/cw-optimizoor)
[![Documentation](https://docs.rs/cw-optimizoor/badge.svg)](https://docs.rs/cw-optimizoor)
![License](https://img.shields.io/crates/l/cw-optimizoor.svg)

A blazingly fast alternative to [CosmWasm/rust-optimizer] for compiling and optimizing CosmWasm smart contracts.

It is primarily meant to speed up local development and testing.

## Features

- Fast, especially in workspaces with many contracts
- Uses the same optimization strategy as `rust-optimizer` by default
- No Docker dependency
- Supports single-contract and multi-contract workspaces
- Ships as a [cargo subcommand]
- Cross-platform and cross-architecture

## Installation

```sh
# Prerequisites
$ rustup toolchain install 1.85.0
$ rustup target add wasm32-unknown-unknown

$ cargo install --locked cw-optimizoor
```

You might also need some native build tooling:

### Linux

```sh
# Ubuntu
$ sudo apt install -y cmake pkg-config libssl-dev clang

# Fedora
$ sudo dnf install -y cmake openssl-devel clang
```

### macOS

```sh
$ xcode-select --install
$ brew install cmake pkg-config
```

### Windows

- [Visual Studio C++, Clang & Build Tools]
- [CMake]
- [Python]

## Usage

```sh
$ cargo cw-optimizoor --help

Usage: cargo cw-optimizoor [WORKSPACE_PATH]

Arguments:
  [WORKSPACE_PATH]  Path to the workspace dir or Cargo.toml

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Example

```sh
$ cargo cw-optimizoor              # defaults to the current dir
# cargo cw-optimizoor Cargo.toml   # point directly at a workspace manifest
# cargo cw-optimizoor .            # or use the current directory explicitly

🧐️  Compiling .../monorepo/Cargo.toml
    Finished release [optimized] target(s) in 0.10s

🤓  Intermediate checksums:
    ...326a37596ef54377869d8f7caa37cec393333b9808c9ecc75ddadf1357193a50  contract_1.wasm
    ...170190ce817c36aa093263f4689abaffafe363909aea13e48b80c43a39a7cde9  contract_2.wasm
    ...6a718777f28b2e213e3f18f60ffbf62febe563072e8a89b0cfa5359b3e0bed1b  contract_3.wasm
    ...9f9dae24e8a388730b40de3092117cf84476dacfb6ed0112bec53b1b21127333  contract_4.wasm
    ...9255c18758fd0b27de38c8aacd2030167b9d3c1575374d811f89742be8af4f8b  contract_5.wasm

🥸  Ahh I'm optimiziing
    ...✅ contract_1 was optimized.
    ...⏭️ contract_2 is unchanged. Skipping.
    ...✅ contract_3 was optimized.
    ...⏭️ contract_4 is unchanged. Skipping.
    ...✅ contract_5 was optimized.

🤓  Final checksums:
    ...e11db2d5b9ff3e14deee2a04ee40be0d1f8da96c4a45bc55348ea74ff4a4d4ae  contract_1-aarch64.wasm
    ...0565368394fd2fa1409909f63fe11d09f37a1f777f26bc5ddb65d17c2fc82bb9  contract_2-aarch64.wasm
    ...1364e024dab8cc057d090d8686042d8ab5e41e810b16d464be71a24aedc79ad3  contract_3-aarch64.wasm
    ...4f553da8e620137c194eddfddcaa7baa29239ec723d0b1b2b49d11fe625986e5  contract_4-aarch64.wasm
    ...61ea8988f4275c15785d7496c453a37ae4c3b021d4521120fc5c0d532287f864  contract_5-aarch64.wasm

🫡  Done. Saved optimized artifacts to:
   .../monorepo/artifacts
```

## Testing

```sh
$ git submodule update --init --recursive
$ cargo test --lib
$ cargo test --test integration
$ cargo test --test e2e
$ cargo test --doc
```

The integration and e2e suites copy the `tests/cw-plus` fixture into `target/tests/...` before running, so checked-in fixtures stay clean and all generated artifacts stay out of the repo.

## Releases

Releases are prepared from `master` with `release-plz`, and tags continue to use the `v0.x.y` format. Changelog history lives in [CHANGELOG.md](CHANGELOG.md).

[CosmWasm/rust-optimizer]: https://github.com/CosmWasm/rust-optimizer
[cargo subcommand]: https://doc.rust-lang.org/cargo/reference/external-tools.html#custom-subcommands
[Visual Studio C++, Clang & Build Tools]: https://visualstudio.microsoft.com/downloads/
[CMake]: https://cmake.org/download/
[Python]: https://www.python.org/downloads/windows/
