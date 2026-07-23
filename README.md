# Small test app for various rust9x functionality

1. Update the lib paths in `.cargo/config.toml` to match yours. If you want to build for a system
   that is already supported natively by your default msvc installation, you can remove these or
   comment them out.
2. `cargo +rust9x build --target i686-rust9x-windows-pc` or equivalent

- Make sure to put `hello.exe` next to the executable for testing.
- If you plan on testing on Windows 9x/ME, make sure to place `unicows.dll` next to the executable
  - For stack unwinding, also place `dbghelp.dll` (version 5.1 or higher, see r9x wiki for more info)
- If you link dynamically, make sure to provide the necessary runtime libraries.
