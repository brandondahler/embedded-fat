# Development

## Prerequisites
* Rust stable toolchain
  * If unsure how to install, see [rustup](https://rustup.rs/)
* Cargo Clippy
  * `rustup component add clippy`
* Cargo LLVM Tools
  * `rustup component add llvm-tools`
* [Optional] Java
  * Used for Gradle orchestration of cargo commands

## Building
Building works like every other Cargo-based rust project -- you can use the default commands:
```shell
cargo fmt
cargo clippy
cargo build
cargo test
cargo clean
```

There are also convenience orchestrations implement in Gradle:
```shell
# fmt, clippy, build, test (with coverage)
./gradlew devBuild

# fmt --check, clippy, build for each feature combo, test for each important feature combo (with coverage)
./gradlew build
```

## Unicode Casing Codegen
The FAT filesystems use codepages for short filenames and UTF-16 for long filenames, but in both are case-insensitive. 
In order to optimize code size while maintaining high-performance, a custom code-generated implementation of case 
folding is maintained under `./unicode_casing_codegen`.  This codegen project takes in the official Unicode 
`CaseFolding.txt` file and produces an optimized implementation at `./src/encoding/unicode/case_folding{,tests}.rs`.

The codegen can be re-run on the latest CodeFolding.txt file by running:

```shell
./gradlew :unicode_casing_codegen:regenerateUnicodeCasing
```