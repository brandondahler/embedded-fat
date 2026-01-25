# embedded-fat
A FAT filesystem implementation optimized for embedded systems.

**Maintainer note: This project is still in active development and is not released anywhere.**

## Features
| Name                   | Description                                                                                                    | Default | Code Impact                                                                                                                                                                                                                                                                                                                       |
|------------------------|----------------------------------------------------------------------------------------------------------------|---------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `async`                | Adds support for the async API                                                                                 | Enabled | Disabling shrinks the dependency tree and reduces the total code required, this may improve compilation performance if unused.                                                                                                                                                                                                    |
| `sync`                 | Adds support for the sync API                                                                                  | Enabled | Disabling reduces total code required, this may slightly improve compilation performance if unused.                                                                                                                                                                                                                               |
| `unicode-case-folding` | Enables support for non-ASCII case insensitivity when attempting to find an existing directory or file entries | Enabled | Disabling will reduce the binary size by up to 4KB and improve exact case directory/file matching performance by up to 3x at the cost of no longer supporting non-ASCII case insensitivity.  This may consequently write directory or file entries in a standards non-conforming manner -- disable this feature at your own risk. |

## License
Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
