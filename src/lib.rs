//! Terminal I/O kernel — raw mode, ANSI escapes, event reading, image rendering.
//!
//! Terminal-specific face of the text layout stack. The general text
//! layout kernel (Ruler, Box, AFM pdf-base14) lives in `Ptyktos/ruler` and
//! is re-exported here for backward compat with code that uses
//! `textfold::{Box, Ruler}`.

#![forbid(unsafe_code)]

// Re-export the kernel so existing `textfold::Box` / `textfold::Ruler`
// imports keep working.
pub use ruler::{Box, Ruler};

pub mod crossterm;
pub mod image;
pub mod ratatui_compat;
pub mod terminal;
