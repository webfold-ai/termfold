//! True per-pixel fidelity — the B-throne applicative: emit the actual pixels, no cell codebook.
//!
//! Two terminal graphics protocols, both fed the decoded RGBA directly:
//!   * **kitty** ([`kitty`]) — raw 24-bit RGB, base64'd: **full fidelity**, every pixel exact
//!     (kitty / WezTerm / Ghostty / Konsole).
//!   * **sixel** ([`sixel`]) — per-pixel but palette-bound; here a 6×6×6 + grays cube (the cone
//!     SELECT at K≈232), broad support (xterm / mlterm / foot / WezTerm / DEC).
//!
//! These are the *full-frame* fidelity path. The **O(Δ) video** path is the half-block cell-diff in
//! the parent module ([`super::flush_diff`]) — re-emit only changed cells. Graphics protocols re-send
//! a frame (kitty can place/delete by id for inter-frame work); choose half-block for motion, kitty
//! for a still at full fidelity.

use super::Frame;

// ── base64 (zero-dep) — kitty transmits its pixel payload base64-encoded ──
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(data: &[u8], out: &mut Vec<u8>) {
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8 | data[i + 2] as u32;
        out.push(B64[(n >> 18 & 63) as usize]);
        out.push(B64[(n >> 12 & 63) as usize]);
        out.push(B64[(n >> 6 & 63) as usize]);
        out.push(B64[(n & 63) as usize]);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(B64[(n >> 18 & 63) as usize]);
        out.push(B64[(n >> 12 & 63) as usize]);
        out.extend_from_slice(b"==");
    } else if rem == 2 {
        let n = (data[i] as u32) << 16 | (data[i + 1] as u32) << 8;
        out.push(B64[(n >> 18 & 63) as usize]);
        out.push(B64[(n >> 12 & 63) as usize]);
        out.push(B64[(n >> 6 & 63) as usize]);
        out.push(b'=');
    }
}

/// Encode `frame` as the **kitty graphics protocol** — raw 24-bit RGB, **full fidelity** (every
/// source pixel exact). Chunked at 4096 base64 bytes (`m=1` more / `m=0` last), as the protocol
/// requires. Returns the escape-sequence bytes ready to write to the terminal.
pub fn kitty(frame: &Frame) -> Vec<u8> {
    // pack RGB (drop alpha; the parent composites if needed)
    let mut rgb = Vec::with_capacity(frame.w * frame.h * 3);
    for y in 0..frame.h {
        for x in 0..frame.w {
            let i = (y * frame.w + x) * 4;
            rgb.extend_from_slice(&frame.rgba[i..i + 3]);
        }
    }
    let mut b64 = Vec::with_capacity(rgb.len() * 4 / 3 + 4);
    base64(&rgb, &mut b64);

    let mut out = Vec::with_capacity(b64.len() + 256);
    let chunks = b64.chunks(4096);
    let n = chunks.len().max(1);
    for (k, chunk) in b64.chunks(4096).enumerate() {
        out.extend_from_slice(b"\x1b_G");
        if k == 0 {
            // a=T transmit+display, f=24 RGB, s=width v=height
            out.extend_from_slice(b"a=T,f=24,s=");
            push_num(&mut out, frame.w);
            out.extend_from_slice(b",v=");
            push_num(&mut out, frame.h);
            out.extend_from_slice(b",m=");
            out.push(if n > 1 { b'1' } else { b'0' });
        } else {
            out.extend_from_slice(b"m=");
            out.push(if k + 1 < n { b'1' } else { b'0' });
        }
        out.push(b';');
        out.extend_from_slice(chunk);
        out.extend_from_slice(b"\x1b\\");
    }
    out
}

fn push_num(out: &mut Vec<u8>, mut v: usize) {
    if v == 0 {
        out.push(b'0');
        return;
    }
    let mut d = [0u8; 20];
    let mut n = 0;
    while v > 0 {
        d[n] = b'0' + (v % 10) as u8;
        v /= 10;
        n += 1;
    }
    for i in (0..n).rev() {
        out.push(d[i]);
    }
}

// ── sixel — palette-bound full-pixel graphics (the cone SELECT at a fixed cube) ──

/// The fixed display cube: 6×6×6 RGB (216) + 16 grays = 232 colors — the cone codebook K≈232.
fn cube_index(r: u8, g: u8, b: u8) -> u8 {
    let q = |c: u8| (c as u16 * 5 / 255) as u8; // 0..5
    let (qr, qg, qb) = (q(r), q(g), q(b));
    qr * 36 + qg * 6 + qb // 0..215
}

fn cube_rgb(i: u8) -> (u8, u8, u8) {
    if i < 216 {
        let r = i / 36;
        let g = (i % 36) / 6;
        let b = i % 6;
        let s = |c: u8| (c as u16 * 255 / 5) as u8;
        (s(r), s(g), s(b))
    } else {
        let g = ((i - 216) as u16 * 255 / 15) as u8;
        (g, g, g)
    }
}

/// Encode `frame` as **sixel** with the fixed 232-color cube. Bands of 6 rows; per band, one pass
/// per color present, RLE-compressed. Broad terminal support; color is cube-quantized (not 24-bit).
pub fn sixel(frame: &Frame) -> Vec<u8> {
    let (w, h) = (frame.w, frame.h);
    // map every pixel to a cube index once (the SELECT)
    let mut idx = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            idx[y * w + x] = cube_index(frame.rgba[i], frame.rgba[i + 1], frame.rgba[i + 2]);
        }
    }
    let mut out = Vec::with_capacity(w * h / 4 + 1024);
    out.extend_from_slice(b"\x1bPq"); // sixel start
                                      // palette: only register the colors actually used
    let mut used = [false; 256];
    for &c in &idx {
        used[c as usize] = true;
    }
    for c in 0..256u16 {
        if used[c as usize] {
            let (r, g, b) = cube_rgb(c as u8);
            out.extend_from_slice(b"#");
            push_num(&mut out, c as usize);
            out.extend_from_slice(b";2;");
            push_num(&mut out, (r as usize * 100) / 255);
            out.push(b';');
            push_num(&mut out, (g as usize * 100) / 255);
            out.push(b';');
            push_num(&mut out, (b as usize * 100) / 255);
        }
    }
    let mut band = 0;
    while band * 6 < h {
        let y0 = band * 6;
        let rows = (h - y0).min(6);
        for c in 0..256u16 {
            if !used[c as usize] {
                continue;
            }
            // does this color appear in this band? build the sixel row
            let mut row = vec![0u8; w];
            let mut any = false;
            for x in 0..w {
                let mut bits = 0u8;
                for dy in 0..rows {
                    if idx[(y0 + dy) * w + x] == c as u8 {
                        bits |= 1 << dy;
                    }
                }
                if bits != 0 {
                    any = true;
                }
                row[x] = bits;
            }
            if !any {
                continue;
            }
            out.push(b'#');
            push_num(&mut out, c as usize);
            // RLE the sixel bytes
            let mut x = 0;
            while x < w {
                let v = row[x];
                let mut run = 1;
                while x + run < w && row[x + run] == v {
                    run += 1;
                }
                if run >= 3 {
                    out.push(b'!');
                    push_num(&mut out, run);
                    out.push(0x3F + v);
                } else {
                    for _ in 0..run {
                        out.push(0x3F + v);
                    }
                }
                x += run;
            }
            out.push(b'$'); // graphics CR — next color overwrites same band
        }
        out.push(b'-'); // graphics NL — next band
        band += 1;
    }
    out.extend_from_slice(b"\x1b\\"); // ST
    out
}

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
    fn base64_known_vector() {
        let mut out = Vec::new();
        base64(b"Man", &mut out);
        assert_eq!(out, b"TWFu");
        out.clear();
        base64(b"Ma", &mut out);
        assert_eq!(out, b"TWE=");
        out.clear();
        base64(b"M", &mut out);
        assert_eq!(out, b"TQ==");
    }

    #[test]
    fn kitty_full_fidelity_envelope() {
        let rgba = solid(4, 3, 200, 100, 50);
        let k = kitty(&Frame::new(4, 3, &rgba));
        assert!(k.starts_with(b"\x1b_G"));
        assert!(k.ends_with(b"\x1b\\"));
        assert!(k.windows(7).any(|w| w == b"f=24,s="), "24-bit RGB declared");
        // 4×3×3 = 36 RGB bytes → 48 base64 chars, one chunk
    }

    #[test]
    fn sixel_envelope_and_palette() {
        let rgba = solid(6, 6, 255, 0, 0);
        let s = sixel(&Frame::new(6, 6, &rgba));
        assert!(s.starts_with(b"\x1bPq"));
        assert!(s.ends_with(b"\x1b\\"));
        assert!(
            s.windows(3).any(|w| w == b";2;"),
            "a palette color is defined"
        );
    }

    #[test]
    fn cube_roundtrip_is_close() {
        let i = cube_index(255, 0, 0);
        let (r, g, b) = cube_rgb(i);
        assert!(r > 200 && g < 30 && b < 30);
    }
}
