# Integration & E2E Tests

The integration and e2e tests run against a checked-in [CosmWasm/cw-plus] fixture and use [wabt] to verify the output artifacts.

The e2e tests use [cucumber-rs]. Each scenario copies the `tests/cw-plus` fixture into `target/tests/workspaces/...` first, so the submodule stays read-only and all generated artifacts stay under `target/tests/...`.

### Setup
```sh
$ brew install wabt
$ git submodule update --init --recursive
```

### Run
```sh
$ cargo test --test integration
$ cargo test --test e2e
$ cargo test --doc

# or both
$ cargo test --test '*'
```

[CosmWasm/cw-plus]: https://github.com/CosmWasm/cw-plus
[wabt]: https://github.com/WebAssembly/wabt
[cucumber-rs]: https://github.com/cucumber-rs/cucumber
