# Integration Tests

The integration tests run against [CosmWasm/cw-plus] and use [wabt] to verify the outputs. 

### Setup
```sh
$ brew install wabt
$ git submodule update --init
```

### Run
```sh
$ cargo run test
```

[CosmWasm/cw-plus]: https://github.com/CosmWasm/cw-plus
[wabt]: https://github.com/WebAssembly/wabt
