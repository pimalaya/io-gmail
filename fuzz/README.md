# Fuzzing

Coverage-guided fuzzing of the batch answer parser with [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer).

The `batch` target answers a `GmailBatch` of one to eight calls with a `multipart/mixed` body assembled from fuzzed pieces (delimiters, close delimiters, `Content-ID` lines, status lines, line breaks and raw bytes) under a fuzzed boundary. Its oracle is that whatever a server sends, the coroutine answers every call exactly once, in the order of the calls, with no status exactly when the answer is missing, never panics and never asks for a read after EOF.

cargo-fuzz needs a nightly toolchain, which the `fuzz` devShell of the repository flake carries along with cargo-fuzz itself. Bound a run with the libFuzzer flags after the `--` separator:

```sh
nix develop .#fuzz --command bash -c "cargo fuzz run batch -- -max_total_time=60"
```

New interesting inputs land in fuzz/corpus/batch/ and crashes in fuzz/artifacts/batch/ (both gitignored), from where a crash is replayed with `cargo fuzz run batch fuzz/artifacts/batch/<file>`.
