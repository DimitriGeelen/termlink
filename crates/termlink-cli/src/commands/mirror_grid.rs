use std::io::{self, Write};

use unicode_width::UnicodeWidthChar;
use vte::{Params, Perform};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SgrState {
    pub fg: Option<u8>,
    pub bg: Option<u8>,
    pub bold: bool,
    pub underline: bool,
    pub reverse: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cell {
    pub ch: char,
    pub sgr: SgrState,
    /// Display width: 1 = normal, 2 = wide (left half), 0 = wide-cell continuation (right half).
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', sgr: SgrState::default(), width: 1 }
    }
}

pub(crate) struct Grid {
    pub cols: u16,
    pub rows: u16,
    pub cells: Vec<Cell>,
    pub cursor_row: u16,
    pub cursor_col: u16,
    sgr: SgrState,
    saved_cursor: Option<(u16, u16)>,
    unhandled_csi: u64,
    scroll_top: u16,
    scroll_bottom: u16,
    pub cursor_visible: bool,
    /// Saved primary-screen state when alt-screen is active.
    alt_backup: Option<AltScreenBackup>,
    /// Previous rendered frame — `None` means the next render must be a full paint.
    prev_cells: Option<Vec<Cell>>,
    prev_cursor: (u16, u16),
    prev_cursor_visible: bool,
}

struct AltScreenBackup {
    cells: Vec<Cell>,
    cursor_row: u16,
    cursor_col: u16,
    sgr: SgrState,
}

impl Grid {
    pub fn new(cols: u16, rows: u16) -> Self {
        let size = (cols as usize) * (rows as usize);
        Grid {
            cols,
            rows,
            cells: vec![Cell::default(); size],
            cursor_row: 0,
            cursor_col: 0,
            sgr: SgrState::default(),
            saved_cursor: None,
            unhandled_csi: 0,
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            cursor_visible: true,
            alt_backup: None,
            prev_cells: None,
            prev_cursor: (0, 0),
            prev_cursor_visible: true,
        }
    }

    #[cfg(test)]
    pub fn is_alt_screen(&self) -> bool {
        self.alt_backup.is_some()
    }

    fn scroll_up_region(&mut self) {
        let top = self.scroll_top as usize;
        let bot = self.scroll_bottom as usize;
        if top >= bot || bot >= self.rows as usize {
            return;
        }
        let cols = self.cols as usize;
        for r in top..bot {
            let dst_start = r * cols;
            let src_start = (r + 1) * cols;
            for c in 0..cols {
                self.cells[dst_start + c] = self.cells[src_start + c];
            }
        }
        let last = bot * cols;
        for c in 0..cols {
            self.cells[last + c] = Cell::default();
        }
    }

    fn scroll_down_region(&mut self) {
        let top = self.scroll_top as usize;
        let bot = self.scroll_bottom as usize;
        if top >= bot || bot >= self.rows as usize {
            return;
        }
        let cols = self.cols as usize;
        for r in (top + 1..=bot).rev() {
            let dst_start = r * cols;
            let src_start = (r - 1) * cols;
            for c in 0..cols {
                self.cells[dst_start + c] = self.cells[src_start + c];
            }
        }
        let first = top * cols;
        for c in 0..cols {
            self.cells[first + c] = Cell::default();
        }
    }

    fn enter_alt_screen(&mut self) {
        if self.alt_backup.is_some() {
            return;
        }
        let size = (self.cols as usize) * (self.rows as usize);
        self.alt_backup = Some(AltScreenBackup {
            cells: std::mem::replace(&mut self.cells, vec![Cell::default(); size]),
            cursor_row: self.cursor_row,
            cursor_col: self.cursor_col,
            sgr: self.sgr,
        });
        self.cursor_row = 0;
        self.cursor_col = 0;
        self.sgr = SgrState::default();
    }

    fn leave_alt_screen(&mut self) {
        if let Some(backup) = self.alt_backup.take() {
            self.cells = backup.cells;
            self.cursor_row = backup.cursor_row;
            self.cursor_col = backup.cursor_col;
            self.sgr = backup.sgr;
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if cols == self.cols && rows == self.rows {
            return;
        }
        let size = (cols as usize) * (rows as usize);
        let mut new_cells = vec![Cell::default(); size];
        let copy_cols = cols.min(self.cols) as usize;
        let copy_rows = rows.min(self.rows) as usize;
        for r in 0..copy_rows {
            for c in 0..copy_cols {
                let old_idx = r * self.cols as usize + c;
                let new_idx = r * cols as usize + c;
                new_cells[new_idx] = self.cells[old_idx];
            }
        }
        self.cols = cols;
        self.rows = rows;
        self.cells = new_cells;
        if self.cursor_row >= rows {
            self.cursor_row = rows.saturating_sub(1);
        }
        if self.cursor_col >= cols {
            self.cursor_col = cols.saturating_sub(1);
        }
        self.scroll_top = 0;
        self.scroll_bottom = rows.saturating_sub(1);
    }

    #[inline]
    fn idx(&self, row: u16, col: u16) -> usize {
        (row as usize) * (self.cols as usize) + (col as usize)
    }

    fn put_char(&mut self, ch: char) {
        if self.cursor_row >= self.rows {
            self.cursor_row = self.rows.saturating_sub(1);
        }
        let w = UnicodeWidthChar::width(ch).unwrap_or(1) as u8;
        if w == 0 {
            // Combining mark / zero-width — silently drop for v1.
            return;
        }
        if self.cursor_col.saturating_add(w as u16) > self.cols {
            // Wrap to next line.
            self.cursor_col = 0;
            if self.cursor_row == self.scroll_bottom {
                self.scroll_up_region();
            } else {
                self.cursor_row = self.cursor_row.saturating_add(1).min(self.rows.saturating_sub(1));
            }
        }
        let idx = self.idx(self.cursor_row, self.cursor_col);
        if let Some(cell) = self.cells.get_mut(idx) {
            cell.ch = ch;
            cell.sgr = self.sgr;
            cell.width = w;
        }
        if w == 2 {
            // Mark continuation cell so diff render can skip it.
            let cont = self.idx(self.cursor_row, self.cursor_col + 1);
            if let Some(cell) = self.cells.get_mut(cont) {
                cell.ch = '\0';
                cell.sgr = self.sgr;
                cell.width = 0;
            }
        }
        self.cursor_col = self.cursor_col.saturating_add(w as u16);
    }

    fn clear_line(&mut self, mode: u16) {
        let row = self.cursor_row;
        let (from, to) = match mode {
            1 => (0, self.cursor_col), // to cursor (inclusive handled loosely)
            2 => (0, self.cols),       // entire line
            _ => (self.cursor_col, self.cols),
        };
        for c in from..to.min(self.cols) {
            let idx = self.idx(row, c);
            if let Some(cell) = self.cells.get_mut(idx) {
                *cell = Cell::default();
            }
        }
    }

    fn clear_display(&mut self, mode: u16) {
        match mode {
            1 => {
                for r in 0..self.cursor_row {
                    for c in 0..self.cols {
                        let idx = self.idx(r, c);
                        self.cells[idx] = Cell::default();
                    }
                }
                self.clear_line(1);
            }
            2 | 3 => {
                for cell in self.cells.iter_mut() {
                    *cell = Cell::default();
                }
            }
            _ => {
                self.clear_line(0);
                for r in (self.cursor_row.saturating_add(1))..self.rows {
                    for c in 0..self.cols {
                        let idx = self.idx(r, c);
                        self.cells[idx] = Cell::default();
                    }
                }
            }
        }
    }

    fn apply_sgr(&mut self, params: &Params) {
        let iter = params.iter().flat_map(|slice| slice.iter().copied());
        let mut collected: Vec<u16> = iter.collect();
        if collected.is_empty() {
            collected.push(0);
        }
        let mut i = 0;
        while i < collected.len() {
            let p = collected[i];
            match p {
                0 => self.sgr = SgrState::default(),
                1 => self.sgr.bold = true,
                4 => self.sgr.underline = true,
                7 => self.sgr.reverse = true,
                22 => self.sgr.bold = false,
                24 => self.sgr.underline = false,
                27 => self.sgr.reverse = false,
                30..=37 => self.sgr.fg = Some((p - 30) as u8),
                38 => {
                    if i + 1 < collected.len()
                        && collected[i + 1] == 5
                        && i + 2 < collected.len()
                    {
                        self.sgr.fg = Some(collected[i + 2] as u8);
                        i += 2;
                    }
                }
                39 => self.sgr.fg = None,
                40..=47 => self.sgr.bg = Some((p - 40) as u8),
                48 => {
                    if i + 1 < collected.len()
                        && collected[i + 1] == 5
                        && i + 2 < collected.len()
                    {
                        self.sgr.bg = Some(collected[i + 2] as u8);
                        i += 2;
                    }
                }
                49 => self.sgr.bg = None,
                90..=97 => self.sgr.fg = Some((p - 90 + 8) as u8),
                100..=107 => self.sgr.bg = Some((p - 100 + 8) as u8),
                _ => {}
            }
            i += 1;
        }
    }

    /// Emit a full repaint to `out`. Always safe, always correct.
    pub fn render_full(&mut self, out: &mut impl Write) -> io::Result<()> {
        // Clear viewer screen + home cursor + reset SGR.
        out.write_all(b"\x1b[2J\x1b[H\x1b[0m")?;
        let mut last_sgr = SgrState::default();
        for r in 0..self.rows {
            out.write_all(format!("\x1b[{};1H", r + 1).as_bytes())?;
            let mut c = 0u16;
            while c < self.cols {
                let cell = self.cells[self.idx(r, c)];
                if cell.width == 0 {
                    // Continuation cell — already emitted by the wide predecessor.
                    c += 1;
                    continue;
                }
                if cell.sgr != last_sgr {
                    write_sgr(out, &cell.sgr)?;
                    last_sgr = cell.sgr;
                }
                let mut buf = [0u8; 4];
                out.write_all(cell.ch.encode_utf8(&mut buf).as_bytes())?;
                c += cell.width.max(1) as u16;
            }
        }
        out.write_all(
            format!("\x1b[0m\x1b[{};{}H", self.cursor_row + 1, self.cursor_col + 1).as_bytes(),
        )?;
        out.write_all(if self.cursor_visible { b"\x1b[?25h" } else { b"\x1b[?25l" })?;
        out.flush()?;
        self.snapshot_as_prev();
        Ok(())
    }

    /// Emit only cells that differ from the last render. Falls back to `render_full`
    /// on first call (prev buffer empty) or if dims changed since the last snapshot.
    pub fn render_diff(&mut self, out: &mut impl Write) -> io::Result<()> {
        let need_full = match &self.prev_cells {
            None => true,
            Some(prev) => prev.len() != self.cells.len(),
        };
        if need_full {
            return self.render_full(out);
        }
        let prev = self.prev_cells.as_ref().unwrap();
        let mut wrote_any = false;
        for r in 0..self.rows {
            let mut c = 0u16;
            while c < self.cols {
                let idx = self.idx(r, c);
                let cur = self.cells[idx];
                let old = prev[idx];
                if cur.width == 0 {
                    c += 1;
                    continue;
                }
                if cur != old {
                    // Find run of dirty cells on this row.
                    let start = c;
                    out.write_all(format!("\x1b[{};{}H", r + 1, start + 1).as_bytes())?;
                    // Force SGR re-emit on first cell of the run by seeding a differing state.
                    let mut last_sgr = SgrState { bold: !cur.sgr.bold, ..cur.sgr };
                    while c < self.cols {
                        let i = self.idx(r, c);
                        let cc = self.cells[i];
                        let oo = prev[i];
                        if cc.width == 0 {
                            c += 1;
                            continue;
                        }
                        if cc == oo {
                            break;
                        }
                        if cc.sgr != last_sgr {
                            write_sgr(out, &cc.sgr)?;
                            last_sgr = cc.sgr;
                        }
                        let mut buf = [0u8; 4];
                        out.write_all(cc.ch.encode_utf8(&mut buf).as_bytes())?;
                        c += cc.width.max(1) as u16;
                        wrote_any = true;
                    }
                } else {
                    c += cur.width.max(1) as u16;
                }
            }
        }
        // Cursor & visibility.
        let cur_pos = (self.cursor_row, self.cursor_col);
        if wrote_any || cur_pos != self.prev_cursor {
            out.write_all(
                format!("\x1b[0m\x1b[{};{}H", self.cursor_row + 1, self.cursor_col + 1).as_bytes(),
            )?;
        }
        if self.cursor_visible != self.prev_cursor_visible {
            out.write_all(if self.cursor_visible { b"\x1b[?25h" } else { b"\x1b[?25l" })?;
        }
        out.flush()?;
        self.snapshot_as_prev();
        Ok(())
    }

    fn snapshot_as_prev(&mut self) {
        self.prev_cells = Some(self.cells.clone());
        self.prev_cursor = (self.cursor_row, self.cursor_col);
        self.prev_cursor_visible = self.cursor_visible;
    }

    /// Viewport-aware full repaint. Does NOT clear the host screen (would wipe
    /// sibling panels). Emits every cell into the rectangle at
    /// (offset_row, offset_col). Offsets are 0-based; CUP is 1-based per ANSI.
    pub fn render_full_at(
        &mut self,
        offset_row: u16,
        offset_col: u16,
        out: &mut impl Write,
    ) -> io::Result<()> {
        let mut last_sgr = SgrState::default();
        out.write_all(b"\x1b[0m")?;
        for r in 0..self.rows {
            out.write_all(
                format!("\x1b[{};{}H", offset_row + r + 1, offset_col + 1).as_bytes(),
            )?;
            let mut c = 0u16;
            while c < self.cols {
                let cell = self.cells[self.idx(r, c)];
                if cell.width == 0 {
                    c += 1;
                    continue;
                }
                if cell.sgr != last_sgr {
                    write_sgr(out, &cell.sgr)?;
                    last_sgr = cell.sgr;
                }
                let mut buf = [0u8; 4];
                out.write_all(cell.ch.encode_utf8(&mut buf).as_bytes())?;
                c += cell.width.max(1) as u16;
            }
        }
        out.write_all(b"\x1b[0m")?;
        out.flush()?;
        self.snapshot_as_prev();
        Ok(())
    }

    /// Viewport-aware dirty-cell diff. Does NOT move the host cursor at end —
    /// the composite renderer decides final cursor placement (typically on the
    /// focused panel). Falls back to `render_full_at` on first call.
    pub fn render_diff_at(
        &mut self,
        offset_row: u16,
        offset_col: u16,
        out: &mut impl Write,
    ) -> io::Result<()> {
        let need_full = match &self.prev_cells {
            None => true,
            Some(prev) => prev.len() != self.cells.len(),
        };
        if need_full {
            return self.render_full_at(offset_row, offset_col, out);
        }
        let prev = self.prev_cells.as_ref().unwrap();
        for r in 0..self.rows {
            let mut c = 0u16;
            while c < self.cols {
                let idx = self.idx(r, c);
                let cur = self.cells[idx];
                let old = prev[idx];
                if cur.width == 0 {
                    c += 1;
                    continue;
                }
                if cur != old {
                    let start = c;
                    out.write_all(
                        format!(
                            "\x1b[{};{}H",
                            offset_row + r + 1,
                            offset_col + start + 1
                        )
                        .as_bytes(),
                    )?;
                    let mut last_sgr = SgrState { bold: !cur.sgr.bold, ..cur.sgr };
                    while c < self.cols {
                        let i = self.idx(r, c);
                        let cc = self.cells[i];
                        let oo = prev[i];
                        if cc.width == 0 {
                            c += 1;
                            continue;
                        }
                        if cc == oo {
                            break;
                        }
                        if cc.sgr != last_sgr {
                            write_sgr(out, &cc.sgr)?;
                            last_sgr = cc.sgr;
                        }
                        let mut buf = [0u8; 4];
                        out.write_all(cc.ch.encode_utf8(&mut buf).as_bytes())?;
                        c += cc.width.max(1) as u16;
                    }
                } else {
                    c += cur.width.max(1) as u16;
                }
            }
        }
        out.write_all(b"\x1b[0m")?;
        out.flush()?;
        self.snapshot_as_prev();
        Ok(())
    }
}

fn write_sgr(out: &mut impl Write, s: &SgrState) -> io::Result<()> {
    out.write_all(b"\x1b[0")?;
    if s.bold {
        out.write_all(b";1")?;
    }
    if s.underline {
        out.write_all(b";4")?;
    }
    if s.reverse {
        out.write_all(b";7")?;
    }
    if let Some(fg) = s.fg {
        if fg < 8 {
            out.write_all(format!(";{}", 30 + fg).as_bytes())?;
        } else if fg < 16 {
            out.write_all(format!(";{}", 90 + (fg - 8)).as_bytes())?;
        } else {
            out.write_all(format!(";38;5;{}", fg).as_bytes())?;
        }
    }
    if let Some(bg) = s.bg {
        if bg < 8 {
            out.write_all(format!(";{}", 40 + bg).as_bytes())?;
        } else if bg < 16 {
            out.write_all(format!(";{}", 100 + (bg - 8)).as_bytes())?;
        } else {
            out.write_all(format!(";48;5;{}", bg).as_bytes())?;
        }
    }
    out.write_all(b"m")
}

impl Perform for Grid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            0x09 => {
                let next_tab = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next_tab.min(self.cols.saturating_sub(1));
            }
            0x0A..=0x0C => {
                if self.cursor_row == self.scroll_bottom {
                    self.scroll_up_region();
                } else {
                    self.cursor_row = self.cursor_row.saturating_add(1).min(self.rows.saturating_sub(1));
                }
            }
            0x0D => {
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        // Reject private-mode markers except the DEC ones we care about — bail as unhandled for now.
        let first = params
            .iter()
            .next()
            .and_then(|s| s.first().copied())
            .unwrap_or(0);
        let second = params
            .iter()
            .nth(1)
            .and_then(|s| s.first().copied())
            .unwrap_or(0);
        match c {
            'A' => {
                let n = first.max(1);
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                let n = first.max(1);
                self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
            }
            'C' => {
                let n = first.max(1);
                self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
            }
            'D' => {
                let n = first.max(1);
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                let r = first.max(1).saturating_sub(1);
                let col = second.max(1).saturating_sub(1);
                self.cursor_row = r.min(self.rows.saturating_sub(1));
                self.cursor_col = col.min(self.cols.saturating_sub(1));
            }
            'J' => {
                self.clear_display(first);
            }
            'K' => {
                self.clear_line(first);
            }
            'm' => {
                self.apply_sgr(params);
            }
            's' => {
                self.saved_cursor = Some((self.cursor_row, self.cursor_col));
            }
            'u' => {
                if let Some((r, c)) = self.saved_cursor {
                    self.cursor_row = r;
                    self.cursor_col = c;
                }
            }
            'r' => {
                // DECSTBM: set scroll region. Params are 1-based; default 1..=rows.
                let top = first.max(1).saturating_sub(1);
                let bot = if second == 0 {
                    self.rows.saturating_sub(1)
                } else {
                    second.saturating_sub(1).min(self.rows.saturating_sub(1))
                };
                if top < bot {
                    self.scroll_top = top;
                    self.scroll_bottom = bot;
                }
                self.cursor_row = top;
                self.cursor_col = 0;
            }
            'S' => {
                let n = first.max(1);
                for _ in 0..n {
                    self.scroll_up_region();
                }
            }
            'T' => {
                let n = first.max(1);
                for _ in 0..n {
                    self.scroll_down_region();
                }
            }
            'h' | 'l' if intermediates == [b'?'] => {
                let set = c == 'h';
                for slice in params.iter() {
                    for &mode in slice.iter() {
                        match mode {
                            25 => self.cursor_visible = set,
                            1049 => {
                                if set {
                                    self.enter_alt_screen();
                                } else {
                                    self.leave_alt_screen();
                                }
                            }
                            _ => self.unhandled_csi = self.unhandled_csi.saturating_add(1),
                        }
                    }
                }
            }
            _ => {
                let _ = intermediates;
                self.unhandled_csi = self.unhandled_csi.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vte::Parser;

    fn feed(grid: &mut Grid, bytes: &[u8]) {
        let mut parser = Parser::new();
        for b in bytes {
            parser.advance(grid, *b);
        }
    }

    #[test]
    fn plain_text_lands_on_grid() {
        let mut g = Grid::new(10, 2);
        feed(&mut g, b"hello");
        assert_eq!(g.cells[0].ch, 'h');
        assert_eq!(g.cells[4].ch, 'o');
        assert_eq!(g.cursor_col, 5);
    }

    #[test]
    fn cup_moves_cursor() {
        let mut g = Grid::new(10, 5);
        feed(&mut g, b"\x1b[3;4HX");
        // Row index 2, col index 3 → offset 2*10+3 = 23
        assert_eq!(g.cells[23].ch, 'X');
    }

    #[test]
    fn el_0_clears_right() {
        let mut g = Grid::new(8, 2);
        feed(&mut g, b"abcdefgh\x1b[1;4H\x1b[0K");
        assert_eq!(g.cells[0].ch, 'a');
        assert_eq!(g.cells[2].ch, 'c');
        assert_eq!(g.cells[3].ch, ' ');
        assert_eq!(g.cells[7].ch, ' ');
    }

    #[test]
    fn sgr_red_applies_fg() {
        let mut g = Grid::new(4, 1);
        feed(&mut g, b"\x1b[31mR\x1b[0m");
        assert_eq!(g.cells[0].ch, 'R');
        assert_eq!(g.cells[0].sgr.fg, Some(1));
    }

    #[test]
    fn render_full_emits_bytes() {
        let mut g = Grid::new(3, 1);
        feed(&mut g, b"\x1b[31mAB\x1b[0m");
        let mut out = Vec::new();
        g.render_full(&mut out).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("A"));
        assert!(s.contains("B"));
        assert!(s.contains("\x1b[2J"));
    }

    #[test]
    fn decset_1049_swaps_alt_screen() {
        let mut g = Grid::new(4, 2);
        feed(&mut g, b"P1");
        assert!(!g.is_alt_screen());
        feed(&mut g, b"\x1b[?1049h");
        assert!(g.is_alt_screen());
        feed(&mut g, b"ALT");
        // Alt buffer has ALT, primary buffer untouched.
        assert_eq!(g.cells[0].ch, 'A');
        feed(&mut g, b"\x1b[?1049l");
        assert!(!g.is_alt_screen());
        assert_eq!(g.cells[0].ch, 'P');
        assert_eq!(g.cells[1].ch, '1');
    }

    #[test]
    fn decset_25_toggles_cursor_visibility() {
        let mut g = Grid::new(2, 1);
        assert!(g.cursor_visible);
        feed(&mut g, b"\x1b[?25l");
        assert!(!g.cursor_visible);
        feed(&mut g, b"\x1b[?25h");
        assert!(g.cursor_visible);
    }

    #[test]
    fn decstbm_plus_lf_scrolls_region() {
        let mut g = Grid::new(2, 4);
        // Fill rows 0..=3 with 'a'/'b'/'c'/'d'.
        feed(&mut g, b"aa\r\nbb\r\ncc\r\ndd");
        // Set scroll region rows 2..=3 (1-based: 2;3), cursor to (top, 0).
        feed(&mut g, b"\x1b[2;3r");
        // Move cursor to bottom of region (1-based row 3, col 1).
        feed(&mut g, b"\x1b[3;1H");
        // LF at bottom of region should scroll 'bb' up to row 1, row 2 becomes ''.
        feed(&mut g, b"\n");
        // Row 0 (untouched by scroll): 'aa'
        assert_eq!(g.cells[0].ch, 'a');
        // Row 1: was 'bb' before, scroll up replaced it with 'cc'
        assert_eq!(g.cells[2].ch, 'c');
        // Row 2 (new bottom inside region): cleared
        assert_eq!(g.cells[4].ch, ' ');
        // Row 3 (outside region): still 'dd'
        assert_eq!(g.cells[6].ch, 'd');
    }

    #[test]
    fn render_emits_cursor_visibility() {
        let mut g = Grid::new(2, 1);
        feed(&mut g, b"\x1b[?25l");
        let mut out = Vec::new();
        g.render_full(&mut out).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("\x1b[?25l"));
        assert!(!s.contains("\x1b[?25h"));
    }

    #[test]
    fn render_diff_emits_only_changes() {
        let mut g = Grid::new(4, 1);
        feed(&mut g, b"ABCD");
        // First call primes prev_cells via render_full.
        let mut out1 = Vec::new();
        g.render_diff(&mut out1).unwrap();
        assert!(out1.windows(4).any(|w| w == b"\x1b[2J"), "first render should be full repaint");

        // Change only cell 1 from 'B' to 'X'.
        feed(&mut g, b"\x1b[1;2HX");
        let mut out2 = Vec::new();
        g.render_diff(&mut out2).unwrap();
        let s = String::from_utf8_lossy(&out2);
        // No full clear on second render.
        assert!(!s.contains("\x1b[2J"), "second render must not full-clear");
        // X is emitted.
        assert!(s.contains("X"));
        // A/C/D are NOT re-emitted as standalone chars (we only wrote the changed cell).
        // Sanity: diff output is smaller than full output for a 1-cell change.
        assert!(out2.len() < out1.len(), "diff output should be smaller than full");
    }

    #[test]
    fn render_diff_at_applies_offset() {
        let mut g = Grid::new(3, 1);
        feed(&mut g, b"ABC");
        // First call primes prev_cells via render_full_at.
        let mut out1 = Vec::new();
        g.render_diff_at(10, 20, &mut out1).unwrap();
        let s1 = String::from_utf8_lossy(&out1);
        // CUP should be offset by (10, 20): row 11 col 21 (1-based).
        assert!(s1.contains("\x1b[11;21H"), "full-at should CUP to offset row/col: got {:?}", s1);
        // No host-screen clear — composite viewport must not wipe siblings.
        assert!(!s1.contains("\x1b[2J"));

        feed(&mut g, b"\x1b[1;2HX");
        let mut out2 = Vec::new();
        g.render_diff_at(10, 20, &mut out2).unwrap();
        let s2 = String::from_utf8_lossy(&out2);
        // Diff at offset: change at cell (0,1) maps to CUP(11, 22).
        assert!(s2.contains("\x1b[11;22H"), "diff-at should offset dirty run CUP: got {:?}", s2);
        assert!(s2.contains("X"));
    }

    #[test]
    fn wide_char_advances_cursor_by_two() {
        let mut g = Grid::new(6, 1);
        // CJK '中' is width-2.
        let mut parser = Parser::new();
        for b in "A中B".as_bytes() {
            parser.advance(&mut g, *b);
        }
        assert_eq!(g.cells[0].ch, 'A');
        assert_eq!(g.cells[0].width, 1);
        assert_eq!(g.cells[1].ch, '中');
        assert_eq!(g.cells[1].width, 2);
        assert_eq!(g.cells[2].width, 0, "cell to the right of wide char is a continuation");
        assert_eq!(g.cells[3].ch, 'B');
        assert_eq!(g.cursor_col, 4);
    }
}
