# vjfind

A search engine and CLI for finding VJ clips by the `{tags}` you write in folder and file names.
A GUI will come later. See [docs/spec.md](docs/spec.md) for the details.

```
vjfind --root 'D:\clips' query 'opening #solo #3D|#2D -#self-shot'
vjfind --root 'D:\clips' tags
vjfind --root 'D:\clips' lint
```

## Development

```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
