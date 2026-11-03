# embedded-fat
A FAT filesystem implementation optimized for embedded systems.

**Maintainer note: This project is still in active development and is not released anywhere.**

## Package Features
| Name                   | Description                                                                                                    | Default | Code Impact                                                                                                                                                                                                                                                                                                                       |
|------------------------|----------------------------------------------------------------------------------------------------------------|---------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `async`                | Adds support for the async API                                                                                 | Enabled | Disabling shrinks the dependency tree and reduces the total code required, this may improve compilation performance if disabled.                                                                                                                                                                                                  |
| `sync`                 | Adds support for the sync API                                                                                  | Enabled | Disabling reduces total code required, this may slightly improve compilation performance if disabled.                                                                                                                                                                                                                             |
| `unicode-case-folding` | Enables support for non-ASCII case insensitivity when attempting to find an existing directory or file entries | Enabled | Disabling will reduce the binary size by up to 1KB and improve exact case directory/file matching performance by up to 2x at the cost of no longer supporting non-ASCII case insensitivity.  This may consequently write directory or file entries in a standards non-conforming manner -- disable this feature at your own risk. |

## Example Usage
```rust
let file_system = FileSystemBuilder::from_stream(StdFile::new(
    File::open("disks/fat32.bin").unwrap(),
  ))
  .build()
  .unwrap();

let mut file = file_system
  .open("foo.txt")
  .unwrap();

let mut bytes = [0; 5];
file.read_exact(&mut bytes).unwrap();
```

## Block-level Access
Block-level access is implemented through the stream instance passed into the FileSystemBuilder. Streams must implement 
`Read + Seek` to support read access, with `Write` additionally required for write access. Both synchronous traits from 
`embedded-io` and asyncrhonous traits from `embedded-io-async` are supported

The `FileSystem` and `File` instances expose the corresponding synchronous and/or asynchronous methods based on the 
underlying stream's capabilities.

## Async Support
Embedded systems often interact directly with hardware peripherals, which can perform I/O operations independently of 
the CPU. Asynchronous block-level access allows these operations to proceed without blocking the calling task, allowing
other work to be performed while waiting for I/O to complete.

This is particularly useful when working with hardware-assisted I/O, such as DMA transfers, hardware SPI peripherals, 
and programmable I/O peripherals, which can handle portions of the data transfer independently of the CPU.

Asynchronous I/O operations are managed through a bounded queue, allowing multiple operations to remain outstanding 
concurrently. The maximum number of outstanding operations is configurable through `FileSystemBuilder`, providing 
control over concurrency and memory usage without requiring dynamic allocation.

See the [RTIC](https://rtic.rs) and [Embassy](https://embassy.dev/) projects for frameworks that provide async runtimes
and concurrency primitives suitable for embedded systems.

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Development

### Building
Gradle is used to orchestrate various build sub-commands.  You don't have to use Gradle to run the various cargo 
commands -- it just helps automate the various flows.

Common build commands:
* Build during development -- formats, runs clippy, runs tests, and builds with default features 
  ```shell
  ./gradlew devBuild
  ```
* Build all configuration combinations -- checks formatting, runs clippy, runs tests, and builds all feature 
  combinations
  ```shell
  ./gradlew build
  ```
* Reformat all code
  ```shell
  ./gradlew fmt
  ```
* Delete all build artifacts
  ```shell
  ./gradlew clean
  ```
* Regenerate Unicode case folding code -- automatically downloads CaseFolding.txt and runs generator
  ```shell
  ./gradlew :unicode_casing_codegen:regenerateUnicodeCasing
  ```
