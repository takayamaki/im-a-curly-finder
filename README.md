# I'm a Curly Finder

A search engine and CLI for finding VJ clips by the `{tags}` you write in folder and file names.
A GUI will come later. See [docs/spec.md](docs/spec.md) for the details.

```
curly --root 'D:\clips' query 'opening #solo #3D|#2D -#self-shot'
curly --root 'D:\clips' tags
curly --root 'D:\clips' lint
```

## Development

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### Building for Windows (from Linux / WSL)

Cross-compile to `x86_64-pc-windows-gnu` with mingw-w64.
The exe only needs the DLLs that come with Windows, so a single `curly.exe` is enough.

```
sudo apt install mingw-w64   # if not installed yet
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# → target/x86_64-pc-windows-gnu/release/curly.exe
```

We avoid the msvc target (cargo-xwin). It uses Microsoft's CRT and Windows SDK,
whose license expects them to be used together with Visual Studio.
