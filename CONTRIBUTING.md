# Contributing to termfold

Thanks for helping. This is a small crate, so the bar is simple: the change should build, pass the
checks below, and say what it does and why.

## Before you start

- For anything larger than a bug fix, open an issue first so the approach can be agreed.
- Security problems: do not open an issue. Follow SECURITY.md.
- By contributing you agree that your contribution is licensed under the MIT OR Apache-2.0 terms in
  `LICENSE-MIT` and `LICENSE-APACHE`.

## Local checks

The Rust toolchain must be at least the `rust-version` in `Cargo.toml` (1.88). CI runs the first five
commands on exactly 1.88.0. The fuzz crate is built by the fuzz workflow.

```bash
cargo fmt --all -- --check
cargo build --locked --all-targets
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo doc --locked --no-deps
cargo check --manifest-path fuzz/Cargo.toml
```

`cargo deny check advisories bans licenses sources` and `cargo audit` run in CI as well.

## Rules for changes

- **No `unsafe`.** `src/lib.rs` has `#![forbid(unsafe_code)]`.
- **Lockfile.** Commit `Cargo.lock` changes with the dependency change. Use `--locked` locally.
- **Dependencies.** Keep the set small. A git dependency must be pinned with `rev = "<full sha>"`
  and its URL must appear in `deny.toml` under `allow-git`.
- **Parser changes** (`terminal::parse_event` and helpers) must not panic or overflow on any byte
  sequence. Add a case to `tests/parse_robustness.rs` for any input that exposed a problem.
- **Public API.** Anything `pub` is a commitment. Say in the PR when a change alters it.
- **Terminal behaviour.** If you change escape sequences or raw-mode handling, say which terminal
  emulators and platforms you tested. The code currently shells out to GNU `stty -F`.

## Pull requests

Fill in the template. Keep one logical change per PR. CI must pass before review.
