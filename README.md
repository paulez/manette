# Manette

Manette is a terminal file explorer. It allows you to navigate the file though its full screen terminal UI and run commands from its shell.

## Run

## Development

Manette is built on Rust, so you will need [rustc](https://www.rust-lang.org/tools/install) and [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to compile it from source.

### Cargo run

```
git clone https://github.com/paulez/manette.git
cd manette
cargo run
```

### Cargo release

We build a release inside a container for maximum compatibility.

```
podman run --rm -v "$PWD":/usr/src/manette:Z -w /usr/src/manette rust cargo build --release
```
