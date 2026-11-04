# Changelog

All notable changes to this project are documented in this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/) once it reaches 1.0.

## [Unreleased]

### Fixed

- The default build did not compile: `bitflags` was optional, but `terminal::KeyModifiers` needs it
  unconditionally. `bitflags` is now a required dependency.
- `crossterm.rs` referred to a non-existent `textfold` crate path; the paths now point at `termfold`.
- Parsing of CSI and SGR-mouse numbers overflowed (a panic in debug builds) on long digit runs.
  Accumulation now saturates, and mouse coordinates are clamped to `u16`.
- Clippy: `io::Error::new(ErrorKind::Other, ..)` replaced by `io::Error::other`.

### Changed

- `ruler` is now a git dependency on `https://github.com/webfold-ai/ruler`, pinned to
  `a37d96c7aeaf688fa912f28243a16a32558fac64`. It previously pointed at a private GitLab host.
- `terminal::parse_event` is now `pub` (documented) so it can be fuzzed and tested.
- Removed the `ratatui` feature. The `ratatui` crate was not a dependency, so the feature could not
  build.
- `rust-version` is now declared as `1.88`.

### Removed from the build

- `src/image.rs`, `src/image/graphics.rs` and `src/ratatui_compat.rs` are no longer compiled. They
  import a `tui` module that is not in this repository, and `ratatui_compat` needs the `ratatui`
  crate. The files are kept in the tree for reference. See README.md.

### Added

- Licences (`LICENSE-MIT`, `LICENSE-APACHE`), `SECURITY.md`, `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `CONTRIBUTORS.md`.
- CI, CodeQL, cargo-fuzz, supply-chain (cargo-audit, cargo-deny, CycloneDX SBOM), Scorecard,
  dependency-review and release workflows. Every action is pinned to a commit SHA.
- `deny.toml`, `fuzz/` (target `fuzz_parse_event`), `tests/parse_robustness.rs`.

## [0.1.0]

Initial import, extracted from the Ptyktos fold monorepo (README history).
