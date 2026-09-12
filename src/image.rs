//! Full-fidelity image/video display in the terminal — the cell as a pixel-block fit.
//!
//! Lifting chafa/viu (`sang adapt`) the display is one operation: **fit each terminal cell to its
//! pixel block**, the trinity of `NeuralPrimitives.model_is_three_faces` (`sang model image --tri`):
//!   * **NB** — the glyph/codebook: which symbol carves the cell. Here the half-block `▀` (K=2: a
//!     top region and a bottom region), so one cell carries **two vertical pixels**.
//!   * **B** — the colors: the foreground (top) and background (bottom) the two regions take, as
//!     24-bit truecolor — no palette quantize, so this is full fidelity at cell granularity.
//!   * **B/U** — the fit: each output pixel is the **box-average** of its source block (the B/U
//!     accumulator over the block), the scale baked into the read.
//!
//! Sixel/kitty (true per-pixel fidelity) is the B-throne applicative — [`graphics`].
//!
//! **O(Δ) video.** A frame is written into a [`Buffer`](super::tui::Buffer); the next frame diffs
//! against it ([`Buffer::diff`](super::tui::Buffer::diff)) and [`flush_diff`] emits **only the
//! changed cells** — the streaming law over the Web of cells (each cell a coordinate; re-emit the
//! flaw, never the rest). A static region between frames costs nothing.

use super::tui::{Buffer, Cell, Color, Modifier, Rect};
use std::io::{self, Write};

/// A decoded RGBA frame (4 bytes/pixel, row-major) — what transcode's `png`/`jpeg`/`h264`/`av1`
/// decoders produce. Display is decoder-agnostic: any source that fills this is shown.
pub struct Frame<'a> {
    pub w: usize,
    pub h: usize,
    pub rgba: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn new(w: usize, h: usize, rgba: &'a [u8]) -> Self {
        Frame { w, h, rgba }
    }

    /// One source pixel, alpha composited over black (RGBA → RGB). Out-of-range → black.
    #[inline]
    fn px(&self, x: usize, y: usize) -> (u32, u32, u32) {
        if x >= self.w || y >= self.h {
            return (0, 0, 0);
        }
        let i = (y * self.w + x) * 4;
        let a = self.rgba[i + 3] as u32;
        // composite over black: c·a/255
        (
            (self.rgba[i] as u32 * a) / 255,
            (self.rgba[i + 1] as u32 * a) / 255,
            (self.rgba[i + 2] as u32 * a) / 255,
        )
    }

    /// Box-average the source block mapping to output pixel `(ox, oy)` in an `ow × oh` grid — the
    /// B/U accumulator (sum over the block, divide by count). The scale, integer, exact.
    fn box_avg(&self, ox: usize, oy: usize, ow: usize, oh: usize) -> (u8, u8, u8) {
        let x0 = ox * self.w / ow;
        let x1 = (((ox + 1) * self.w + ow - 1) / ow).max(x0 + 1).min(self.w);
        let y0 = oy * self.h / oh;
        let y1 = (((oy + 1) * self.h + oh - 1) / oh).max(y0 + 1).min(self.h);
        let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
        for sy in y0..y1 {
            for sx in x0..x1 {
                let (pr, pg, pb) = self.px(sx, sy);
                r += pr;
                g += pg;
                b += pb;
                n += 1;
            }
        }
        if n == 0 {
            return (0, 0, 0);
        }
        ((r / n) as u8, (g / n) as u8, (b / n) as u8)
    }
}

/// The cell aspect: a terminal cell is ~twice as tall as wide, and half-block packs 2 px/cell
/// vertically, so the pixel grid is `cols × (2·rows)` and is square-ish per pixel.
pub const VPX_PER_CELL: usize = 2;

/// Fit `(w, h)` into `cols × (2·rows)` preserving aspect — the cell area the image occupies.
pub fn fit(w: usize, h: usize, cols: u16, rows: u16) -> (u16, u16) {
    if w == 0 || h == 0 || cols == 0 || rows == 0 {
        return (0, 0);
    }
    let avail_px_w = cols as f64;
    let avail_px_h = (rows as usize * VPX_PER_CELL) as f64;
    let scale = (avail_px_w / w as f64).min(avail_px_h / h as f64);
    let ow = ((w as f64 * scale).round() as usize)
        .max(1)
        .min(cols as usize);
    let oh_px = ((h as f64 * scale).round() as usize)
        .max(1)
        .min(rows as usize * VPX_PER_CELL);
    let oh = oh_px.div_ceil(VPX_PER_CELL);
    (ow as u16, oh as u16)
}

/// Render a frame into `area` of `buf` as half-block cells (`▀`, fg=top px, bg=bottom px), 24-bit
/// truecolor — full fidelity at cell granularity. Aspect-preserved within `area`. O(cells).
pub fn render_halfblock(frame: &Frame, buf: &mut Buffer, area: Rect) {
    let (cols, rows) = fit(frame.w, frame.h, area.w, area.h);
    let (ow, oh_cells) = (cols as usize, rows as usize);
    let oh = oh_cells * VPX_PER_CELL; // pixel rows
    for cy in 0..oh_cells {
        for cx in 0..ow {
            let top = frame.box_avg(cx, cy * VPX_PER_CELL, ow, oh);
            let bot = frame.box_avg(cx, cy * VPX_PER_CELL + 1, ow, oh);
            buf.set(
                area.x + cx as u16,
                area.y + cy as u16,
                Cell {
                    c: '▀',
                    fg: Color::Rgb(top.0, top.1, top.2),
                    bg: Color::Rgb(bot.0, bot.1, bot.2),
                    modifiers: Modifier::empty(),
                },
            );
        }
    }
}

// ── the truecolor flush (the gap: terminal.rs is 256-color only) ──

fn put_color(out: &mut Vec<u8>, c: Color, fg: bool) {
    let lead: &[u8] = if fg { b"38;2;" } else { b"48;2;" };
    match c {
        Color::Rgb(r, g, b) => {
            out.extend_from_slice(lead);
            push_u8(out, r);
            out.push(b';');
            push_u8(out, g);
            out.push(b';');
            push_u8(out, b);
        }
        Color::Indexed(i) => {
            out.extend_from_slice(if fg { b"38;5;" } else { b"48;5;" });
            push_u8(out, i);
        }
        Color::Reset => out.extend_from_slice(if fg { b"39" } else { b"49" }),
    }
}

#[inline]
fn push_u8(out: &mut Vec<u8>, mut v: u8) {
    if v >= 100 {
        out.push(b'0' + v / 100);
        v %= 100;
        out.push(b'0' + v / 10);
        out.push(b'0' + v % 10);
    } else if v >= 10 {
        out.push(b'0' + v / 10);
        out.push(b'0' + v % 10);
    } else {
        out.push(b'0' + v);
    }
}

fn push_uint(out: &mut Vec<u8>, v: u16) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    let mut digits = [0u8; 5];
    let mut n = 0;
    let mut x = v;
    while x > 0 {
        digits[n] = b'0' + (x % 10) as u8;
        x /= 10;
        n += 1;
    }
    for i in (0..n).rev() {
        out.push(digits[i]);
    }
}

/// Emit one cell at `(x, y)` (1-based screen coords) with truecolor fg+bg into `out`.
fn emit_cell(out: &mut Vec<u8>, x: u16, y: u16, cell: &Cell) {
    out.extend_from_slice(b"\x1b[");
    push_uint(out, y + 1);
    out.push(b';');
    push_uint(out, x + 1);
    out.push(b'H');
    out.extend_from_slice(b"\x1b[");
    put_color(out, cell.fg, true);
    out.push(b';');
    put_color(out, cell.bg, false);
    out.push(b'm');
    let mut cb = [0u8; 4];
    out.extend_from_slice(cell.c.encode_utf8(&mut cb).as_bytes());
}

/// **O(Δ) flush** — emit only the changed cells (from [`Buffer::diff`]) with 24-bit truecolor.
/// For video this is the per-frame cost: the changed cells, never the whole frame. Returns bytes
/// written. (Cursor-addresses each run; a static background is free.)
pub fn flush_diff(diff: &[(u16, u16, &Cell)], out: &mut impl Write) -> io::Result<usize> {
    let mut buf = Vec::with_capacity(diff.len() * 24);
    for &(x, y, cell) in diff {
        emit_cell(&mut buf, x, y, cell);
    }
    buf.extend_from_slice(b"\x1b[0m");
    out.write_all(&buf)?;
    Ok(buf.len())
}

/// Emit a whole buffer (the first frame / full repaint), truecolor, row by row.
pub fn flush_full(buf: &Buffer, out: &mut impl Write) -> io::Result<usize> {
    let mut bytes = Vec::with_capacity(buf.width() as usize * buf.height() as usize * 24);
    bytes.extend_from_slice(b"\x1b[H");
    for y in 0..buf.height() {
        for x in 0..buf.width() {
            emit_cell(&mut bytes, x, y, buf.get(x, y));
        }
        bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"\x1b[0m");
    out.write_all(&bytes)?;
    Ok(bytes.len())
}

pub mod graphics;

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: usize, h: usize, r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 4);
        for _ in 0..w * h {
            v.extend_from_slice(&[r, g, b, 255]);
        }
        v
    }

    #[test]
    fn halfblock_fills_cells_with_truecolor() {
        let rgba = solid(8, 8, 10, 20, 30);
        let frame = Frame::new(8, 8, &rgba);
        let mut buf = Buffer::new(4, 4);
        render_halfblock(
            &frame,
            &mut buf,
            Rect {
                x: 0,
                y: 0,
                w: 4,
                h: 4,
            },
        );
        let c = buf.get(0, 0);
        assert_eq!(c.c, '▀');
        assert_eq!(c.fg, Color::Rgb(10, 20, 30));
        assert_eq!(c.bg, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn box_avg_blends() {
        // left half red, right half blue → averaging the whole into 1 px gives purple-ish
        let mut rgba = Vec::new();
        for _y in 0..2 {
            rgba.extend_from_slice(&[255, 0, 0, 255]);
            rgba.extend_from_slice(&[0, 0, 255, 255]);
        }
        let frame = Frame::new(2, 2, &rgba);
        let avg = frame.box_avg(0, 0, 1, 1);
        assert_eq!(avg, (127, 0, 127));
    }

    #[test]
    fn o_delta_only_changed_cells() {
        // two frames differing in one cell → diff emits exactly one cell
        let a = solid(4, 4, 0, 0, 0);
        let mut prev = Buffer::new(2, 2);
        render_halfblock(
            &Frame::new(4, 4, &a),
            &mut prev,
            Rect {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
            },
        );
        let mut b = solid(4, 4, 0, 0, 0);
        b[0] = 255; // change one source pixel → one top-half cell changes
        let mut cur = Buffer::new(2, 2);
        render_halfblock(
            &Frame::new(4, 4, &b),
            &mut cur,
            Rect {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
            },
        );
        let d = cur.diff(&prev);
        assert_eq!(d.len(), 1, "only the changed cell is in the diff (O(Δ))");
        let mut out = Vec::new();
        let n = flush_diff(&d, &mut out).unwrap();
        assert!(
            n > 0 && out.windows(5).any(|w| w == b"38;2;"),
            "truecolor emitted"
        );
    }

    #[test]
    fn fit_preserves_aspect() {
        // a 100×100 square into an 80×24 terminal (160 px tall): width-bound → 24 cells wide...
        let (c, r) = fit(100, 100, 80, 24);
        // square image, 2px/cell vertical: limited by rows (48 px) vs cols (80) → rows-bound
        assert!(c <= 80 && r <= 24 && c > 0 && r > 0);
        assert_eq!(c as usize, r as usize * VPX_PER_CELL); // square stays square
    }
}
