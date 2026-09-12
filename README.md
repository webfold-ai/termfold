# termfold

> Terminal textfold — the kernel layer that replaces crossterm + chafa + ratatui's render layer.

## What this is

termfold is the **terminal primitive layer** for Ptyktos. Zero external deps. Three things
in one crate:

1. **Ruler** (78K) — the O(1) text layout fold. `prepare()` accumulates cumulative-advance
   prefix sums once; `break_after()` finds line breaks by direct coordinate read into that
   index — O(1), no search. Reflow at a new width re-reads the same ruler; never re-measures.
   The streaming law over text.

2. **terminal** (16K) — raw mode via `stty`, terminal size via ANSI escape, event reading,
   ANSI control sequences. Drop-in replacement for `crossterm`.

3. **image** (11K) — full-fidelity image/video display in the terminal. Cell-as-pixel fit
   (one cell = two vertical pixels via `▀` half-block; full truecolor; box-average over
   the source block — the B/U accumulator). Sixel/kitty for true per-pixel fidelity.

Plus:
- **crossterm** (1.5K) — drop-in shim: change `use crossterm::X` to `use textfold::crossterm::X`.
- **ratatui_compat** (20K) — textfold's TUI types implement ratatui's `Widget` traits.

The high-level widget vocabulary (`Cell`, `Color`, `Modifier`, `Buffer`, `Rect`) is NOT here —
it lives in `Ptyktos/biota/tui` as application code. This crate is just the primitives.

## Why this exists

`crossterm` + `chafa` + `ratatui` would otherwise be three external crates with overlapping
responsibilities and three copies of the terminal-state machinery. The Ptyktos MASQUERADE
doctrine: every external crate replaced by an in-tree primitive.

## Used by

Everything that wants to talk to a terminal. The `a shit load of primitives` that depend
on textfold include `bedrock/src/bin/foundation_dash.rs`, `bedrock/src/bin/marketfold_dash.rs`,
`bedrock/src/bin/servefold.rs`, and every TUI bin in `Ptyktos/biota`.

## Build

```sh
cargo build --release
cargo test
```

## History

- Extracted from `twn/RnD/fold/bedrock/src/textfold/` on 2026-09-12.
- The TUI widget vocabulary (40K `tui.rs`) was separated and lifted to `Ptyktos/biota/tui`.
