# I'm a Curly Finder

Find VJ clips by the `{tags}` you write in folder and file names, and drag them into Resolume or any other app.
See [docs/spec.md](docs/spec.md) for the tag rules and the query syntax.

| Crate | What |
|---|---|
| [crates/curly-finder](crates/curly-finder) | The search engine and the `curly` CLI |
| [crates/curly-gui](crates/curly-gui) | The GUI (Tauri). Type a query, then drag a row into another app |

```
curly --root 'D:\clips' query 'opening #solo #3D|#2D -#self-shot'
curly --root 'D:\clips' tags
curly --root 'D:\clips' lint
curly-gui.exe 'D:\clips'
```

Both take the folder from `CURLY_ROOT` too.

## Development

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

These skip the Tauri app (the `app` feature of curly-gui), so they need no WebView libraries.
Check the app with the Windows target:

```
cargo clippy -p curly-gui --features app --target x86_64-pc-windows-gnu --all-targets -- -D warnings
```

### Building for Windows (from Linux / WSL)

Cross-compile to `x86_64-pc-windows-gnu` with mingw-w64.

```
sudo apt install mingw-w64   # if not installed yet
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu -p curly-gui --features app
```

- `curly.exe` only needs the DLLs that come with Windows.
- `curly-gui.exe` also needs `WebView2Loader.dll`, which the build puts next to it,
  and the WebView2 runtime (part of Windows 11).
  Tauri officially supports only the msvc target, but the gnu build works.

We avoid the msvc target (cargo-xwin). It uses Microsoft's CRT and Windows SDK,
whose license expects them to be used together with Visual Studio.
