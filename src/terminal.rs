//! Terminal raw-mode, event reading, and ANSI control — replaces `crossterm`.
//!
//! Derived from `sang adapt` of crossterm source (B/U 1.28 bits/symbol irreducible).
//! Zero `unsafe`. Raw mode via `stty`. Terminal size via ANSI escape query.
//! Event parsing via standard byte reading. All ANSI escapes are string writes.

use std::io::{self, Read, Write};
use std::time::Duration;

// ── Raw mode via stty ──────────────────────────────────────────────────────

/// Enable raw mode using `stty`. Saves previous state for restoration.
pub fn enable_raw_mode() -> io::Result<TerminalGuard> {
    let prev = std::process::Command::new("stty")
        .args(["-F", "/dev/tty", "-g"])
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("stty -g: {e}")))?;
    let saved = String::from_utf8_lossy(&prev.stdout).trim().to_string();
    if saved.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "stty -g returned empty (not a terminal?)",
        ));
    }
    let status = std::process::Command::new("stty")
        .args(["-F", "/dev/tty", "raw", "-echo", "-isig", "-ixon"])
        .status()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("stty raw: {e}")))?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "stty raw failed"));
    }
    Ok(TerminalGuard { saved })
}

/// Restores terminal settings on drop.
pub struct TerminalGuard {
    saved: String,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = std::process::Command::new("stty")
            .args(["-F", "/dev/tty", &self.saved])
            .status();
    }
}

// ── Terminal size via ANSI escape ──────────────────────────────────────────

/// Get terminal size via `stty size < /dev/tty`. Safer than ANSI escape queries
/// which race with the event reader on stdin.
pub fn size() -> io::Result<(u16, u16)> {
    let out = std::process::Command::new("stty")
        .args(["size", "-F", "/dev/tty"])
        .output()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("stty size: {e}")))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() == 2 {
        if let (Ok(rows), Ok(cols)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
            return Ok((cols, rows));
        }
    }
    Ok((80, 24))
}

// ── Event reading ──────────────────────────────────────────────────────────

/// A terminal input event.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Mouse(MouseEvent),
}

/// A mouse event (SGR 1006 mode). `col`/`row` are 0-indexed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseKind,
    pub col: u16,
    pub row: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseKind {
    Down(u8),
    Up,
    ScrollUp,
    ScrollDown,
    Moved,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    F(u8),
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct KeyModifiers: u8 {
        const NONE = 0;
        const SHIFT = 1;
        const CONTROL = 2;
        const ALT = 4;
    }
}

/// Block until a key event is available. Opens `/dev/stdin` once,
/// reads all available bytes (terminal sends complete key sequences atomically).
pub fn read() -> io::Result<Event> {
    let mut fd = std::fs::File::open("/dev/stdin")?;
    let mut buf = [0u8; 32];
    buf[0] = read_byte(&fd)?;
    let mut n = 1usize;
    // ESC: read the rest of the sequence as one burst. CSI mouse reports
    // (`\x1b[<b;x;yM`, SGR 1006) are longer than arrow keys, so read up to the
    // buffer's end, breaking on a CSI final byte (a letter, `~`, or `R`/`M`/`m`).
    if buf[0] == 0x1b {
        for _ in 0..(buf.len() - 1) {
            if n >= buf.len() {
                break;
            }
            match fd.read(&mut buf[n..n + 1]) {
                Ok(1) => {
                    if buf[n].is_ascii_alphabetic() || buf[n] == b'~' || buf[n] == b'R' {
                        n += 1;
                        break;
                    }
                    n += 1;
                }
                _ => break,
            }
        }
    }
    Ok(parse_event(&buf[..n]))
}

fn read_byte(fd: &std::fs::File) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    // &File implements Read on Unix
    use std::io::Read;
    (&*fd).read_exact(&mut byte)?;
    Ok(byte[0])
}

/// Non-blocking poll: returns true if an event is available within `timeout`.
pub fn poll(timeout: Duration) -> io::Result<bool> {
    // Use select on a stdin reader
    let (tx, rx) = std::sync::mpsc::channel();
    let tx_clone = tx.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 1];
        let result = io::stdin().read_exact(&mut buf);
        let _ = tx_clone.send(result.is_ok());
    });
    match rx.recv_timeout(timeout) {
        Ok(true) => Ok(true),
        Ok(false) => Err(io::Error::new(io::ErrorKind::Other, "stdin read error")),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            // Can't cancel the spawned thread, but it will block on stdin
            // This is a best-effort poll; the thread stays alive until stdin has data
            Ok(false)
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(false),
    }
}

fn parse_event(bytes: &[u8]) -> Event {
    if bytes.is_empty() {
        return Event::Key(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        });
    }
    // Ctrl codes
    if bytes[0] < 0x20 {
        return match bytes[0] {
            0x0a | 0x0d => Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
            }),
            0x08 => Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::NONE,
            }),
            0x09 => Event::Key(KeyEvent {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            }),
            0x1b => {
                if bytes.len() == 1 {
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    })
                } else if bytes.len() >= 2 && bytes[1] == b'[' {
                    parse_csi(&bytes[2..])
                } else if bytes.len() >= 2 {
                    Event::Key(KeyEvent {
                        code: char_code(bytes[1]),
                        modifiers: KeyModifiers::ALT,
                    })
                } else {
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    })
                }
            }
            0x01..=0x1a => {
                let c = (bytes[0] + 0x60) as char;
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::CONTROL,
                })
            }
            _ => Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }),
        };
    }
    if bytes[0] == 0x7f {
        return Event::Key(KeyEvent {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        });
    }
    Event::Key(KeyEvent {
        code: char_code(bytes[0]),
        modifiers: KeyModifiers::NONE,
    })
}

fn parse_csi(params: &[u8]) -> Event {
    // SGR mouse report: `<button;col;row` terminated by `M` (press) or `m` (release).
    if params.first() == Some(&b'<') {
        return parse_mouse_sgr(&params[1..]);
    }
    let mut i = 0;
    let mut nums = Vec::new();
    let mut cur = 0i32;
    while i < params.len() {
        let b = params[i];
        if b == b';' {
            nums.push(cur);
            cur = 0;
            i += 1;
        } else if b.is_ascii_digit() {
            cur = cur * 10 + (b - b'0') as i32;
            i += 1;
        } else {
            nums.push(cur);
            return match b {
                b'A' => event(KeyCode::Up),
                b'B' => event(KeyCode::Down),
                b'C' => event(KeyCode::Right),
                b'D' => event(KeyCode::Left),
                b'H' => event(KeyCode::Home),
                b'F' => event(KeyCode::End),
                b'~' => match nums.first().copied().unwrap_or(0) {
                    2 | 3 => event(KeyCode::Delete),
                    5 => event(KeyCode::PageUp),
                    6 => event(KeyCode::PageDown),
                    15 => event(KeyCode::F(5)),
                    17 => event(KeyCode::F(6)),
                    18 => event(KeyCode::F(7)),
                    19 => event(KeyCode::F(8)),
                    20 => event(KeyCode::F(9)),
                    21 => event(KeyCode::F(10)),
                    23 => event(KeyCode::F(11)),
                    24 => event(KeyCode::F(12)),
                    _ => event(KeyCode::Esc),
                },
                _ => event(KeyCode::Esc),
            };
        }
    }
    event(KeyCode::Esc)
}

/// Parse the body of an SGR mouse report (everything after `\x1b[<`): three
/// semicolon-separated numbers `button;col;row` then a final `M`/`m`. The button
/// code carries scroll in bit 6 (64=up, 65=down); low bits are the click button.
fn parse_mouse_sgr(params: &[u8]) -> Event {
    let mut nums = [0i32; 3];
    let mut idx = 0;
    let mut cur = 0i32;
    let mut last = b'M';
    for &b in params {
        if b == b';' {
            if idx < 3 {
                nums[idx] = cur;
            }
            idx += 1;
            cur = 0;
        } else if b.is_ascii_digit() {
            cur = cur * 10 + (b - b'0') as i32;
        } else {
            last = b; // final byte: 'M' press / 'm' release
            break;
        }
    }
    if idx < 3 {
        nums[idx] = cur;
    }
    let (button, col, row) = (nums[0], nums[1], nums[2]);
    let kind = if button & 0x40 != 0 {
        // wheel: 64 = up, 65 = down
        if button & 0x01 == 0 {
            MouseKind::ScrollUp
        } else {
            MouseKind::ScrollDown
        }
    } else if button & 0x20 != 0 {
        MouseKind::Moved
    } else if last == b'm' {
        MouseKind::Up
    } else {
        MouseKind::Down((button & 0x03) as u8)
    };
    Event::Mouse(MouseEvent {
        kind,
        col: col.saturating_sub(1).max(0) as u16,
        row: row.saturating_sub(1).max(0) as u16,
    })
}

/// Enable mouse reporting (button presses + wheel) in SGR (1006) mode.
pub fn enable_mouse_capture() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?1000h\x1b[?1006h")
}
/// Disable mouse reporting.
pub fn disable_mouse_capture() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?1006l\x1b[?1000l")
}

fn event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
    })
}
fn char_code(b: u8) -> KeyCode {
    KeyCode::Char(b as char)
}

// ── ANSI escape helpers ────────────────────────────────────────────────────

/// Move cursor to column `x`, row `y` (0-indexed).
pub fn move_to(x: u16, y: u16) -> io::Result<()> {
    write!(io::stdout(), "\x1b[{};{}H", y + 1, x + 1)
}
/// Enter alternate screen buffer.
pub fn enter_alternate_screen() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?1049h")
}
/// Leave alternate screen buffer.
pub fn leave_alternate_screen() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?1049l")
}
/// Clear entire screen.
pub fn clear_all() -> io::Result<()> {
    write!(io::stdout(), "\x1b[2J")
}
/// Show cursor.
pub fn show_cursor() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?25h")
}
/// Hide cursor.
pub fn hide_cursor() -> io::Result<()> {
    write!(io::stdout(), "\x1b[?25l")
}
/// Reset all terminal attributes.
pub fn reset_color() -> io::Result<()> {
    write!(io::stdout(), "\x1b[0m")
}
/// Set foreground color (ANSI indexed 0-255).
pub fn set_fg(color: u8) -> io::Result<()> {
    if color < 16 {
        write!(io::stdout(), "\x1b[{}m", 30 + color)
    } else {
        write!(io::stdout(), "\x1b[38;5;{}m", color)
    }
}
/// Set background color.
pub fn set_bg(color: u8) -> io::Result<()> {
    if color < 16 {
        write!(io::stdout(), "\x1b[{}m", 40 + color)
    } else {
        write!(io::stdout(), "\x1b[48;5;{}m", color)
    }
}
/// Set bold attribute.
pub fn set_bold(on: bool) -> io::Result<()> {
    write!(io::stdout(), "\x1b[{}m", if on { 1 } else { 22 })
}
/// Flush stdout.
pub fn flush() -> io::Result<()> {
    io::stdout().flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ctrl_c() {
        assert_eq!(
            parse_event(&[0x03]),
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL
            })
        );
    }
    #[test]
    fn parse_enter() {
        assert_eq!(
            parse_event(&[0x0a]),
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE
            })
        );
    }
    #[test]
    fn parse_escape() {
        assert_eq!(
            parse_event(&[0x1b]),
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE
            })
        );
    }
    #[test]
    fn parse_csi_up() {
        assert_eq!(
            parse_event(&[0x1b, b'[', b'A']),
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                modifiers: KeyModifiers::NONE
            })
        );
    }
    #[test]
    fn parse_alt_key() {
        assert_eq!(
            parse_event(&[0x1b, b'x']),
            Event::Key(KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::ALT
            })
        );
    }

    #[test]
    fn parse_scroll_up() {
        // \x1b[<64;10;20M — wheel up at col 10 row 20
        let seq = b"\x1b[<64;10;20M";
        assert_eq!(
            parse_event(seq),
            Event::Mouse(MouseEvent {
                kind: MouseKind::ScrollUp,
                col: 9,
                row: 19
            })
        );
    }

    #[test]
    fn parse_scroll_down() {
        let seq = b"\x1b[<65;1;1M";
        assert_eq!(
            parse_event(seq),
            Event::Mouse(MouseEvent {
                kind: MouseKind::ScrollDown,
                col: 0,
                row: 0
            })
        );
    }

    #[test]
    fn parse_left_click_and_release() {
        assert_eq!(
            parse_event(b"\x1b[<0;5;7M"),
            Event::Mouse(MouseEvent {
                kind: MouseKind::Down(0),
                col: 4,
                row: 6
            })
        );
        assert_eq!(
            parse_event(b"\x1b[<0;5;7m"),
            Event::Mouse(MouseEvent {
                kind: MouseKind::Up,
                col: 4,
                row: 6
            })
        );
    }
}
