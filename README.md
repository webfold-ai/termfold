# termfold

> **Status: research build.** This code is not production-ready. It is being hardened toward a production release, and interfaces, formats and results may change without notice.

[![CI](https://github.com/webfold-ai/termfold/actions/workflows/ci.yml/badge.svg)](https://github.com/webfold-ai/termfold/actions/workflows/ci.yml)

Terminal input parsing and ANSI output for Rust. Provides Linux raw-mode helpers, keyboard and mouse
event parsing, and escape writers. A compatibility module exposes selected items under
crossterm-shaped paths. Pre-release. This is a supported subset, not a drop-in crossterm or ratatui
replacement.

**Status: 0.1.0, pre-release (`publish = false`).** The API is not stable and the crate is not on
crates.io. Read the limitations below before using it. The crate also re-exports the text-layout
types `Box` and `Ruler` from [`ruler`](https://github.com/webfold-ai/ruler). `#![forbid(unsafe_code)]`
is set in `src/lib.rs`.

## Smallest working example

Parsing needs no terminal, so this runs anywhere. It is compiled and run as a doc test by
`cargo test`.

```rust
use termfold::terminal::{parse_event, Event, KeyCode, KeyModifiers};

// An Up-arrow key press as the terminal sends it: ESC [ A.
match parse_event(b"\x1b[A") {
    Event::Key(key) => {
        assert_eq!(key.code, KeyCode::Up);
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }
    other => panic!("unexpected event: {other:?}"),
}
```

A minimal interactive loop (needs a Linux terminal and GNU `stty`; compiled but not run by the doc
test):

```rust,no_run
use termfold::terminal::{enable_raw_mode, read, Event, KeyCode};

fn main() -> std::io::Result<()> {
    let _guard = enable_raw_mode()?; // restores the terminal settings on drop
    loop {
        if let Event::Key(key) = read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}
```

## Capabilities

| Feature | Status | Notes |
|---|---|---|
| Raw mode (`enable_raw_mode`, `TerminalGuard`) | Works on Linux | Shells out to `stty -F /dev/tty` (GNU coreutils). Not supported on BSD or macOS. |
| Terminal size (`size`) | Works on Linux | `stty size`. Falls back to 80x24 if the output cannot be parsed. |
| Blocking read (`read`) | Works | Reads one key or escape sequence from `/dev/stdin`. |
| Non-blocking poll (`poll`) | Best effort | Spawns a thread that cannot be cancelled; it stays alive until input arrives. |
| Key parsing (`parse_event`) | Works, fuzzed | Single keys, CSI (arrows, Home/End, function and editing keys), Alt-prefixed keys. Unrecognised sequences become `KeyCode::Esc`. |
| Mouse parsing and capture | Works | SGR 1006 reports. Coordinates are 0-based and clamped to `u16`. |
| `Event::Resize` | Not produced | The variant exists, but nothing in the crate emits it. |
| ANSI escape writers | Works | Cursor movement and visibility, alternate screen, clear, 0-255 colour, bold, flush. They write to stdout. |
| `crossterm` module | Path shim | Re-exports `terminal` items under `crossterm::{terminal, event, cursor, style}`. No `execute!` or `queue!`, no command types, no `MouseEvent` or `KeyEventKind`. `enable_raw_mode` returns a guard rather than `()`. |
| `Box`, `Ruler` | Re-export | From `ruler`, pinned in `Cargo.toml`. |
| Windows, macOS | Not supported | Linux only. |
| Image output, ratatui adapters | Not compiled | See "Experimental source (not compiled)". |

## Modules

| Module | Contents |
|---|---|
| `terminal` | The items in the table above |
| `crossterm` | Path shim over `terminal` |
| crate root | `Box` and `Ruler` from `ruler`, pinned to `ruler` rev `a37d96c` |

## Experimental source (not compiled)

These files are in the tree for reference only. They are not in the module tree, are not part of any
build, and are not available features.

- `src/image.rs` and `src/image/graphics.rs`: half-block, kitty and sixel image output.
- `src/ratatui_compat.rs`: ratatui widget adapters.

They import a `tui` module (`Buffer`, `Cell`, `Color`, `Modifier`, `Rect`, `Style`, widget types)
that is not in this repository. `ratatui_compat` also needs the `ratatui` crate, which is not a
dependency. Restoring them needs a decision: vendor the `tui` types here, or depend on the crate that
owns them.

## Platform and behaviour

- `enable_raw_mode`, `size` and the mouse-capture helpers rely on `stty -F /dev/tty` (GNU coreutils
  syntax). This is **tested on Linux only**. BSD and macOS `stty` use different options and are not
  supported by the current code.
- `read()` opens `/dev/stdin` on every call and reads one escape sequence.
- `poll()` starts a thread that reads stdin. The source notes that the thread cannot be cancelled
  and stays alive until input arrives, and the byte it reads is consumed. Treat it as best-effort.
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
