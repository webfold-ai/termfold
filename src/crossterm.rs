//! Crossterm-shaped paths for a selected subset of `termfold::terminal`.
//!
//! This is a path shim, not a drop-in replacement: only the items re-exported below exist, and
//! their signatures follow `termfold::terminal`, not crossterm (for example `enable_raw_mode`
//! returns a guard). There is no `execute!`/`queue!` and no command types.

pub use crate::terminal::{
    clear_all, enable_raw_mode, enter_alternate_screen, flush, hide_cursor, leave_alternate_screen,
    move_to, poll, read, reset_color, set_bg, set_bold, set_fg, show_cursor, size, Event, KeyCode,
    KeyEvent, KeyModifiers, TerminalGuard,
};

/// Crossterm-compatible `terminal` module — re-exports everything under `terminal::`.
pub mod terminal {
    pub use crate::terminal::{
        clear_all, enable_raw_mode, enter_alternate_screen, flush, hide_cursor,
        leave_alternate_screen, move_to, poll, read, reset_color, set_bg, set_bold, set_fg,
        show_cursor, size,
    };
}

/// Crossterm-compatible `event` module — re-exports event types.
pub mod event {
    pub use crate::terminal::{Event, KeyCode, KeyEvent, KeyModifiers};
}

/// Crossterm-compatible `cursor` module — re-exports cursor functions.
pub mod cursor {
    pub use crate::terminal::move_to;
}

/// Crossterm-compatible `style` module — minimal ANSI style support.
pub mod style {
    pub use crate::terminal::{reset_color, set_bg, set_bold, set_fg};
}
