# Contributing guide

Thank you for investing your time in contributing to the I/O Gmail project.

## Development

The development environment is managed by [Nix](https://nixos.org/download.html).
Running `nix-shell` will spawn a shell with everything you need to get started with the lib.

If you do not want to use Nix, you can either use [rustup](https://rust-lang.github.io/rustup/index.html):

```sh
rustup update
```

or install manually the following dependencies:

- [cargo](https://doc.rust-lang.org/cargo/)
- [rustc](https://doc.rust-lang.org/stable/rustc/platform-support.html) (`>= 1.87`)

## Build

```sh
cargo build
```

## Test

```sh
cargo test
```

The test suite is fully offline: each coroutine is driven against a scripted stub stream that replays a canned HTTP response, so no real network access or OAuth token is required.

## Override dependencies

All Pimalaya crates use `[patch.crates-io]` to point to sibling directories.
If you want to build io-gmail against a locally modified dependency (e.g. `io-http`), add the following to `Cargo.toml`:

```toml
[patch.crates-io]
io-http.path = "/path/to/io-http"
```

## Commit style

I/O Gmail follows the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary).
