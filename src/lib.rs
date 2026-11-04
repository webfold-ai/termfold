//! Terminal I/O kernel — raw mode, ANSI escapes, event reading, image rendering.
//!
//! Terminal-specific face of the text layout stack. The general text
//! layout kernel (Ruler, Box, AFM pdf-base14) lives in `Ptyktos/ruler` and
//! is re-exported here for backward compat with code that uses
//! `termfold::{Box, Ruler}`.

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
