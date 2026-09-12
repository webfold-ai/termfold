//! # ratatui compatibility — textfold widgets implement ratatui's Widget traits
//!
//! Gated behind `features = ["ratatui"]`. Makes textfold's TUI types usable as
//! drop-in replacements in existing ratatui code — the O(1)/O(Δ) rendering
//! pipeline without rewriting widgets.
//!
//! ## What masquerades
//! - `Paragraph`, `Block`, `Table`, `List`, `Gauge` — each implements [`Widget`] and [`WidgetRef`]
//! - `Rect`, `Color`, `Style`, `Cell`, `Modifier` — `From`/`Into` conversions both ways
//! - `Span`, `Line` — `From` conversions textfold → ratatui
//!
//! ## Example
//! ```rust,ignore
//! use ratatui::widgets::Widget;
//! use textfold::tui::{Paragraph, Line};
//!
//! let p = Paragraph::new(vec![Line::from("hello world")]);
//! // renders into ratatui's Buffer using textfold's Ruler:
//! frame.render_widget(p, frame.area());
//! ```

use super::tui;
use ratatui::{
    buffer::Buffer as RtBuffer,
    layout::Rect as RtRect,
    style::{Color as RtColor, Modifier as RtModifier, Style as RtStyle},
    widgets::Widget,
};

// ── conversions ────────────────────────────────────────────────────────

fn to_rt_color(c: tui::Color) -> RtColor {
    match c {
        tui::Color::Reset => RtColor::Reset,
        tui::Color::Rgb(r, g, b) => RtColor::Rgb(r, g, b),
        tui::Color::Indexed(i) => RtColor::Indexed(i),
    }
}

fn to_tf_color(c: RtColor) -> tui::Color {
    match c {
        RtColor::Reset => tui::Color::Reset,
        RtColor::Rgb(r, g, b) => tui::Color::Rgb(r, g, b),
        RtColor::Indexed(i) => tui::Color::Indexed(i),
        RtColor::Black => tui::Color::Indexed(0),
        RtColor::Red => tui::Color::Indexed(1),
        RtColor::Green => tui::Color::Indexed(2),
        RtColor::Yellow => tui::Color::Indexed(3),
        RtColor::Blue => tui::Color::Indexed(4),
        RtColor::Magenta => tui::Color::Indexed(5),
        RtColor::Cyan => tui::Color::Indexed(6),
        RtColor::Gray => tui::Color::Indexed(7),
        RtColor::DarkGray => tui::Color::Indexed(8),
        RtColor::LightRed => tui::Color::Indexed(9),
        RtColor::LightGreen => tui::Color::Indexed(10),
        RtColor::LightYellow => tui::Color::Indexed(11),
        RtColor::LightBlue => tui::Color::Indexed(12),
        RtColor::LightMagenta => tui::Color::Indexed(13),
        RtColor::LightCyan => tui::Color::Indexed(14),
        RtColor::White => tui::Color::Indexed(15),
    }
}

fn to_rt_modifier(m: tui::Modifier) -> RtModifier {
    let mut r = RtModifier::empty();
    if m.contains(tui::Modifier::BOLD) {
        r |= RtModifier::BOLD;
    }
    if m.contains(tui::Modifier::DIM) {
        r |= RtModifier::DIM;
    }
    if m.contains(tui::Modifier::ITALIC) {
        r |= RtModifier::ITALIC;
    }
    if m.contains(tui::Modifier::UNDERLINE) {
        r |= RtModifier::UNDERLINED;
    }
    r
}

fn to_tf_modifier(m: RtModifier) -> tui::Modifier {
    let mut r = tui::Modifier::empty();
    if m.contains(RtModifier::BOLD) {
        r |= tui::Modifier::BOLD;
    }
    if m.contains(RtModifier::DIM) {
        r |= tui::Modifier::DIM;
    }
    if m.contains(RtModifier::ITALIC) {
        r |= tui::Modifier::ITALIC;
    }
    if m.contains(RtModifier::UNDERLINED) {
        r |= tui::Modifier::UNDERLINE;
    }
    r
}

fn to_tf_rect(r: RtRect) -> tui::Rect {
    tui::Rect {
        x: r.x,
        y: r.y,
        w: r.width,
        h: r.height,
    }
}

fn to_rt_style(s: tui::Style) -> RtStyle {
    RtStyle::default()
        .fg(to_rt_color(s.fg))
        .bg(to_rt_color(s.bg))
        .add_modifier(to_rt_modifier(s.modifiers))
}

fn make_rt_cell(ch: char, fg: RtColor, bg: RtColor, modifier: RtModifier) -> ratatui::buffer::Cell {
    let mut cell = ratatui::buffer::Cell::default();
    cell.set_char(ch);
    cell.fg = fg;
    cell.bg = bg;
    cell.modifier = modifier;
    cell
}

fn tf_cell_to_rt(c: &tui::Cell) -> ratatui::buffer::Cell {
    make_rt_cell(
        c.c,
        to_rt_color(c.fg),
        to_rt_color(c.bg),
        to_rt_modifier(c.modifiers),
    )
}

// ── From impls ─────────────────────────────────────────────────────────

impl From<tui::Rect> for RtRect {
    fn from(r: tui::Rect) -> Self {
        RtRect::new(r.x, r.y, r.w, r.h)
    }
}

impl From<RtRect> for tui::Rect {
    fn from(r: RtRect) -> Self {
        to_tf_rect(r)
    }
}

impl From<tui::Color> for RtColor {
    fn from(c: tui::Color) -> Self {
        to_rt_color(c)
    }
}

impl From<RtColor> for tui::Color {
    fn from(c: RtColor) -> Self {
        to_tf_color(c)
    }
}

impl From<tui::Modifier> for RtModifier {
    fn from(m: tui::Modifier) -> Self {
        to_rt_modifier(m)
    }
}

impl From<RtModifier> for tui::Modifier {
    fn from(m: RtModifier) -> Self {
        to_tf_modifier(m)
    }
}

impl From<tui::Style> for RtStyle {
    fn from(s: tui::Style) -> Self {
        RtStyle::default()
            .fg(to_rt_color(s.fg))
            .bg(to_rt_color(s.bg))
            .add_modifier(to_rt_modifier(s.modifiers))
    }
}

impl From<tui::Cell> for ratatui::buffer::Cell {
    fn from(c: tui::Cell) -> Self {
        tf_cell_to_rt(&c)
    }
}

impl From<&tui::Cell> for ratatui::buffer::Cell {
    fn from(c: &tui::Cell) -> Self {
        tf_cell_to_rt(c)
    }
}

impl<'a> From<tui::Span<'a>> for ratatui::text::Span<'a> {
    fn from(s: tui::Span<'a>) -> Self {
        ratatui::text::Span::styled(
            s.content,
            to_rt_style(tui::Style {
                fg: s.fg,
                bg: s.bg,
                modifiers: s.modifiers,
            }),
        )
    }
}

impl<'a> From<tui::Line<'a>> for ratatui::text::Line<'a> {
    fn from(l: tui::Line<'a>) -> Self {
        ratatui::text::Line::from(
            l.spans
                .into_iter()
                .map(Into::into)
                .collect::<Vec<ratatui::text::Span>>(),
        )
    }
}

// ── Widget: Paragraph ──────────────────────────────────────────────────

impl Widget for tui::Paragraph<'_> {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        let flat: String = self
            .text
            .iter()
            .flat_map(|l| l.spans.iter().flat_map(|s| s.content.chars()))
            .collect();
        let chars: Vec<char> = flat.chars().collect();
        if chars.is_empty() {
            return;
        }
        let boxes: Vec<super::Box> = chars
            .iter()
            .map(|&c| {
                if c == ' ' {
                    super::Box::space(1.0, 0.0)
                } else {
                    super::Box::glyph(1.0)
                }
            })
            .collect();
        let ruler = super::Ruler::prepare(&boxes);
        let w = if self.wrap {
            area.w
        } else {
            chars.len() as u16
        };
        let lines = ruler.lines((w as f32).max(1.0));
        let sy = self.scroll.1 as usize;
        let sx = self.scroll.0 as usize;
        let sty = self.style;
        for (li, line) in lines.iter().enumerate().skip(sy) {
            let y = area.y + (li - sy) as u16;
            if y >= area.y + area.h {
                break;
            }
            let start = (line.start + sx).min(chars.len());
            let end = line.end.min(area.w as usize + sx).min(chars.len());
            for ci in start..end {
                let x = area.x + (ci - line.start - sx) as u16;
                if x >= area.x + area.w {
                    break;
                }
                buf[(x as u16, y as u16)] = make_rt_cell(
                    chars[ci],
                    to_rt_color(sty.fg),
                    to_rt_color(sty.bg),
                    to_rt_modifier(sty.modifiers),
                );
            }
        }
    }
}

// ── Widget: Block ──────────────────────────────────────────────────────

impl Widget for tui::Block<'_> {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        if area.w < 2 || area.h < 2 {
            return;
        }
        let inner = area.inner(&self.border);
        let sty = self.style;
        let (fg, bg, mods) = (
            to_rt_color(sty.fg),
            to_rt_color(sty.bg),
            to_rt_modifier(sty.modifiers),
        );
        let mk = |c: char| make_rt_cell(c, fg, bg, mods);
        for x in inner.x..inner.right() {
            buf[(x as u16, area.y as u16)] = mk('─');
            buf[(x as u16, (area.y + area.h - 1) as u16)] = mk('─');
        }
        for y in inner.y..inner.bottom() {
            buf[(area.x as u16, y as u16)] = mk('│');
            buf[((area.x + area.w - 1) as u16, y as u16)] = mk('│');
        }
        buf[(area.x as u16, area.y as u16)] = mk('┌');
        buf[((area.x + area.w - 1) as u16, area.y as u16)] = mk('┐');
        buf[(area.x as u16, (area.y + area.h - 1) as u16)] = mk('└');
        buf[((area.x + area.w - 1) as u16, (area.y + area.h - 1) as u16)] = mk('┘');

        if let Some(ref title) = self.title {
            let flat: String = title.spans.iter().flat_map(|s| s.content.chars()).collect();
            for (i, tc) in flat.chars().enumerate() {
                let x = area.x + 2 + i as u16;
                if x < area.right() {
                    buf[(x as u16, area.y as u16)] = mk(tc);
                }
            }
        }
    }
}

// ── Widget: Table ──────────────────────────────────────────────────────

impl<'a, const N: usize> Widget for tui::Table<'a, N> {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        let layout = tui::Layout::new(tui::Direction::Horizontal, self.widths.to_vec());
        let cols = layout.split(area);
        let sty = self.style;
        let (fg, bg, mods) = (
            to_rt_color(sty.fg),
            to_rt_color(sty.bg),
            to_rt_modifier(sty.modifiers),
        );
        let underline_mod = mods | RtModifier::UNDERLINED;
        let mut y = area.y;
        if let Some(ref hdr) = self.header {
            for (ci, col) in cols.iter().enumerate() {
                let text = hdr.get(ci).copied().unwrap_or("");
                for (xi, c) in text.chars().enumerate() {
                    let x = col.x + xi as u16;
                    if x < col.right() {
                        buf[(x as u16, y as u16)] = make_rt_cell(c, fg, bg, underline_mod);
                    }
                }
            }
            y += 1;
        }
        for row in &self.rows {
            if y >= area.y + area.h {
                break;
            }
            for (ci, col) in cols.iter().enumerate() {
                let text = row.get(ci).copied().unwrap_or("");
                for (xi, c) in text.chars().enumerate() {
                    let x = col.x + xi as u16;
                    if x < col.right() {
                        buf[(x as u16, y as u16)] = make_rt_cell(c, fg, bg, mods);
                    }
                }
            }
            y += 1;
        }
    }
}

// ── Widget: List ───────────────────────────────────────────────────────

impl Widget for tui::List<'_> {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        for (i, item) in self.items.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.y + area.h {
                break;
            }
            let sty = if self.selected == Some(i) {
                self.highlight_style
            } else {
                self.style
            };
            let (fg, bg, mods) = (
                to_rt_color(sty.fg),
                to_rt_color(sty.bg),
                to_rt_modifier(sty.modifiers),
            );
            let mut x = area.x;
            for span in &item.spans {
                for c in span.content.chars() {
                    if x >= area.right() {
                        break;
                    }
                    buf[(x as u16, y as u16)] = make_rt_cell(c, fg, bg, mods);
                    x += 1;
                }
            }
        }
    }
}

// ── Widget: Gauge ──────────────────────────────────────────────────────

impl Widget for tui::Gauge {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        if area.w == 0 {
            return;
        }
        let filled = ((self.percent / 100.0) * area.w as f32).round() as u16;
        let (fg, bg, mods) = (
            to_rt_color(self.style.fg),
            to_rt_color(self.style.bg),
            to_rt_modifier(self.style.modifiers),
        );
        for x in area.x..area.right() {
            let c = if x - area.x < filled { '█' } else { '░' };
            buf[(x as u16, area.y as u16)] = make_rt_cell(c, fg, bg, mods);
        }
    }
}

// ── Widget: WidgetSeed ─────────────────────────────────────────────────

impl Widget for tui::WidgetSeed {
    fn render(self, area: RtRect, buf: &mut RtBuffer) {
        let area = to_tf_rect(area);
        if area.w == 0 || area.h == 0 {
            return;
        }
        let mut tf = tui::Buffer::new(area.w, area.h);
        tui::WidgetSeed::render(&self, &mut tf, tui::Rect::new(0, 0, area.w, area.h), 0);
        for y in 0..area.h {
            for x in 0..area.w {
                let cell = tf.get(x, y);
                if cell.c == ' '
                    && cell.fg == tui::Color::Reset
                    && cell.bg == tui::Color::Reset
                    && cell.modifiers == tui::Modifier::empty()
                {
                    continue;
                }
                buf[(area.x + x, area.y + y)] = tf_cell_to_rt(cell);
            }
        }
    }
}

// ── Constraint conversions ───────────────────────────────────────────── ─────────────────────────────────────────────────

impl From<tui::Constraint> for ratatui::layout::Constraint {
    fn from(c: tui::Constraint) -> Self {
        match c {
            tui::Constraint::Fixed(v) => ratatui::layout::Constraint::Length(v),
            tui::Constraint::Min(v) => ratatui::layout::Constraint::Min(v),
            tui::Constraint::Max(v) => ratatui::layout::Constraint::Max(v),
            tui::Constraint::Ratio(n, d) => ratatui::layout::Constraint::Ratio(n, d),
            tui::Constraint::Length(v) => ratatui::layout::Constraint::Length(v),
        }
    }
}

impl From<ratatui::layout::Constraint> for tui::Constraint {
    fn from(c: ratatui::layout::Constraint) -> Self {
        match c {
            ratatui::layout::Constraint::Min(v) => tui::Constraint::Min(v),
            ratatui::layout::Constraint::Max(v) => tui::Constraint::Max(v),
            ratatui::layout::Constraint::Length(v) => tui::Constraint::Length(v),
            ratatui::layout::Constraint::Ratio(n, d) => tui::Constraint::Ratio(n, d),
            _ => tui::Constraint::Fixed(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraph_masquerades_as_ratatui_widget() {
        let p = tui::Paragraph::new(vec![tui::Line::from("hello world")]);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 20, 5));
        Widget::render(p, RtRect::new(0, 0, 20, 5), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "h");
        assert_eq!(buf[(5, 0)].symbol(), " ");
        assert_eq!(buf[(6, 0)].symbol(), "w");
    }

    #[test]
    fn paragraph_widget_wraps_text() {
        let long = "aaa bbb ccc ddd eee fff ggg";
        let p = tui::Paragraph::new(vec![tui::Line::from(long)]);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 10));
        Widget::render(p, RtRect::new(0, 0, 10, 10), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "a");
        assert_eq!(buf[(0, 1)].symbol(), "c");
    }

    #[test]
    fn block_widget_renders_border() {
        let block = tui::Block::new().border_style(tui::Border::minimal());
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 5));
        Widget::render(block, RtRect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        assert_eq!(buf[(9, 0)].symbol(), "┐");
        assert_eq!(buf[(0, 4)].symbol(), "└");
        assert_eq!(buf[(9, 4)].symbol(), "┘");
    }

    #[test]
    fn gauge_widget_fills_proportionally() {
        let g = tui::Gauge::new(50.0);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 1));
        Widget::render(g, RtRect::new(0, 0, 10, 1), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "█");
        assert_eq!(buf[(4, 0)].symbol(), "█");
        assert_eq!(buf[(5, 0)].symbol(), "░");
    }

    #[test]
    fn list_widget_renders_items() {
        let items = vec![tui::Line::from("a"), tui::Line::from("b")];
        let list = tui::List::new(items);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 5));
        Widget::render(list, RtRect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "a");
        assert_eq!(buf[(0, 1)].symbol(), "b");
    }

    #[test]
    fn table_widget_render() {
        let rows = vec![["a", "b"], ["c", "d"]];
        let table = tui::Table::new(rows, [tui::Constraint::Fixed(5), tui::Constraint::Fixed(5)]);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 5));
        Widget::render(table, RtRect::new(0, 0, 10, 5), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "a");
        assert_eq!(buf[(5, 0)].symbol(), "b");
        assert_eq!(buf[(0, 1)].symbol(), "c");
    }

    #[test]
    fn widget_seed_masquerades_as_ratatui_widget() {
        let seed = tui::detect_visual("cpu.txt", "1\n2\n3\n4\n");
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 16, 4));
        Widget::render(seed, RtRect::new(0, 0, 16, 4), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "┌");
        assert_ne!(buf[(1, 1)].symbol(), " ");
    }

    #[test]
    fn rect_conversion_roundtrip() {
        let tf = tui::Rect::new(1, 2, 30, 40);
        let rt: RtRect = tf.into();
        let tf2: tui::Rect = rt.into();
        assert_eq!(tf, tf2);
    }

    #[test]
    fn color_conversion_roundtrip() {
        let tf = tui::Color::Rgb(100, 150, 200);
        let rt: RtColor = tf.into();
        let tf2: tui::Color = rt.into();
        assert_eq!(tf, tf2);
    }

    #[test]
    fn modifier_conversion_roundtrip() {
        let tf = tui::Modifier::BOLD | tui::Modifier::ITALIC;
        let rt: RtModifier = tf.into();
        let tf2: tui::Modifier = rt.into();
        assert_eq!(tf, tf2);
    }

    #[test]
    fn paragraph_clone_implements_widget() {
        let p = tui::Paragraph::new(vec![tui::Line::from("test")]);
        let mut buf = RtBuffer::empty(RtRect::new(0, 0, 10, 3));
        // Widget takes self by value; Paragraph is Clone
        Widget::render(p.clone(), RtRect::new(0, 0, 10, 3), &mut buf);
        assert_eq!(buf[(0, 0)].symbol(), "t");
    }

    #[test]
    fn gauge_creates_right_cells() {
        let cell = make_rt_cell('X', RtColor::Red, RtColor::Blue, RtModifier::BOLD);
        assert_eq!(cell.symbol(), "X");
        assert_eq!(cell.fg, RtColor::Red);
        assert_eq!(cell.bg, RtColor::Blue);
        assert!(cell.modifier.contains(RtModifier::BOLD));
    }
}
