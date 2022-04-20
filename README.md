# Small test app for various rust9x functionality

1. Install `just` (`cargo install just`).
2. Update the lib paths in `.cargo/config.toml` to match yours. If you want to build for a system
   that is already supported natively by your default msvc installation, you can remove these or
   comment them out.
3. Update the variables in the `justfile`. The rust9x toolchain name and `editbin.exe` path (from
   your modern msvc build tools) in particular are important.
4. Use `just build --release` to build using the rust9x toolchain, and regular `cargo` for normal
   toolchain builds.

- Make sure to put `hh3gf.golden.exe` next to the executable for testing.
- If you plan on testing on Windows 9x/ME, make sure to place `unicows.dll` next to the executable
- If you link dynamically, make sure to provide the necessary runtime libraries.

## Credits

`hh3gf.golden.exe` is taken from https://github.com/pts/pts-tinype to test stdout redirection.
