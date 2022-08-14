# Integration & E2E Tests

The integration & e2e tests run against [CosmWasm/cw-plus] and use [wabt] to verify the outputs. 

The e2e tests use [cucumber-rs]. See [features/](features/). 

### Setup
```sh
$ brew install wabt
$ git submodule update --init
```

### Run
```sh
$ cargo test --test integration
$ cargo test --test e2e

# or both
$ cargo test --test '*'
```

[CosmWasm/cw-plus]: https://github.com/CosmWasm/cw-plus
[wabt]: https://github.com/WebAssembly/wabt
[cucumber-rs]: https://github.com/cucumber-rs/cucumber
