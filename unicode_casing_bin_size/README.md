# unicode_casing_bin_size
Allows determining how many bytes the code and data related to the Unicode case folding take up within an executable.

## Process
1. Build without any enabled features
   ```shell
   cargo build --release
   ```
2. Get baseline size
   ```shell
   ls -l ./target/release
   ```
3. Build with the `unicode` feature
   ```shell
   cargo build --release --features unicode
   ```
4. Get the size with the logic included
   ```shell
   ls -l ./target/release
   ```
