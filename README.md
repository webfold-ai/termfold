# termfold

[![CI](https://github.com/Ptyktos/termfold/actions/workflows/ci.yml/badge.svg)](https://github.com/Ptyktos/termfold/actions/workflows/ci.yml)

Terminal I/O for Rust: raw mode, input-event parsing, and ANSI escape writers, with no `unsafe`
code. It re-exports the text-layout types from [`ruler`](https://github.com/webfold-ai/ruler) and
provides a `crossterm`-shaped module path.

**Status: 0.1.0, pre-release (`publish = false`).** The API is not stable and the crate is not on
crates.io. Read the limitations below before using it.

## What is in the crate

| Module | Contents | Notes |
|---|---|---|
| `terminal` | `enable_raw_mode` / `TerminalGuard`, `size`, `read`, `poll`, `parse_event`, `Event` / `KeyEvent` / `KeyCode` / `KeyModifiers` / `MouseEvent`, cursor and colour escape writers, mouse capture (SGR 1006) | Raw mode and size use `stty` |
| `crossterm` | Re-exports of the `terminal` items under crossterm's module names (`terminal`, `event`, `cursor`, `style`) | A path shim, not a crossterm implementation |
| crate root | `Box` and `Ruler`, re-exported from `ruler` | Pinned to `ruler` rev `a37d96c` |

`#![forbid(unsafe_code)]` is set in `src/lib.rs`.

### Not compiled in this snapshot

`src/image.rs`, `src/image/graphics.rs` (half-block, kitty and sixel image output) and
`src/ratatui_compat.rs` (ratatui widget adapters) are **not part of the build**. They import a
`tui` module (`Buffer`, `Cell`, `Color`, `Modifier`, `Rect`, `Style`, widget types) that is not in
this repository. The earlier README placed that module in `Ptyktos/biota`, which this session could
not reach. `ratatui_compat` also needs the `ratatui` crate, which is not a dependency. Those files
are kept for reference. Restoring them needs a decision: vendor the `tui` types here, or depend on
the crate that owns them.

## Platform and behaviour

- `enable_raw_mode`, `size` and the mouse-capture helpers rely on `stty -F /dev/tty` (GNU coreutils
  syntax). This is **tested on Linux only**. BSD and macOS `stty` use different options and are not
  supported by the current code.
- `read()` opens `/dev/stdin` on every call and reads one escape sequence.
- `poll()` starts a thread that reads stdin. The source notes that the thread cannot be cancelled
  and stays alive until input arrives. Treat it as best-effort.
- `parse_event` decodes single-key, CSI (arrows, Home/End, function and editing keys), Alt-prefixed
  keys, and SGR mouse reports. It is `pub` so that tests and fuzzing can reach it. Unrecognised
  sequences become `KeyCode::Esc`.

## Dependencies

- `ruler` (git, pinned by `rev`): the layout kernel. Its own dependency list is outside this
  repository.
- `bitflags` 2 (crates.io): `KeyModifiers`.

`cargo deny` allows only MIT, Apache-2.0 and Unicode-3.0 licences, crates.io, and the single git
source above (`deny.toml`).

## Build and test

```bash
cargo build --locked
cargo test --locked
```

The checks CI runs, on Rust 1.88.0 (the `rust-version` in `Cargo.toml`), are listed in
[CONTRIBUTING.md](CONTRIBUTING.md). The test suite has 8 unit tests in `src/terminal.rs` and 3
regression tests in `tests/parse_robustness.rs`. The regression tests cover integer overflow on
long digit runs and clamping of huge mouse coordinates.

## Fuzzing

`fuzz/` contains one libFuzzer target, `fuzz_parse_event`, which feeds arbitrary bytes to
`terminal::parse_event`. CI runs it for three minutes on each pull request that touches the code, and
for longer on a daily schedule. Build it locally with `cargo check --manifest-path fuzz/Cargo.toml`.
Running it needs nightly and `cargo-fuzz`.

## Security

See [SECURITY.md](SECURITY.md). Report vulnerabilities through GitHub private advisories, not public
issues.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) and
[CONTRIBUTORS.md](CONTRIBUTORS.md). Supply-chain and OpenSSF practices are in `docs/`.

## Licence

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
