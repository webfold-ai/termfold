//! Terminal input parsing and ANSI output: Linux raw-mode helpers, keyboard and mouse event
//! parsing, and escape writers. The `crossterm` module is a path shim over `terminal`, not a
//! crossterm implementation. Pre-release; see README.md for the supported subset and limits.
//!
//! The text layout kernel (`Box`, `Ruler`) lives in `webfold-ai/ruler` and is re-exported here.

#![forbid(unsafe_code)]

// Re-export the kernel so existing `termfold::Box` / `termfold::Ruler`
// imports keep working.
pub use ruler::{Box, Ruler};

// Not compiled in this snapshot: `image`, `image/graphics` and `ratatui_compat` import a `tui`
// module (`Buffer`, `Cell`, `Color`, ...) that is not part of this repository, and
// `ratatui_compat` needs the `ratatui` crate, which is not a dependency. See README.md.
// pub mod image;
// pub mod ratatui_compat;

pub mod crossterm;
pub mod terminal;

// Compile and run the README's Rust examples as doc tests so they cannot go stale.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
