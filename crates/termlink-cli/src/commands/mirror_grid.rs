use std::io::{self, Write};

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
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ', sgr: SgrState::default() }
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
    }

    #[inline]
    fn idx(&self, row: u16, col: u16) -> usize {
        (row as usize) * (self.cols as usize) + (col as usize)
    }

    fn put_char(&mut self, ch: char) {
        if self.cursor_row >= self.rows {
            self.cursor_row = self.rows.saturating_sub(1);
        }
        if self.cursor_col >= self.cols {
            // Wrap to next line.
            self.cursor_col = 0;
            self.cursor_row = self.cursor_row.saturating_add(1).min(self.rows.saturating_sub(1));
        }
        let idx = self.idx(self.cursor_row, self.cursor_col);
        if let Some(cell) = self.cells.get_mut(idx) {
            cell.ch = ch;
            cell.sgr = self.sgr;
        }
        self.cursor_col = self.cursor_col.saturating_add(1);
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
                    if i + 1 < collected.len() && collected[i + 1] == 5 {
                        if i + 2 < collected.len() {
                            self.sgr.fg = Some(collected[i + 2] as u8);
                            i += 2;
                        }
                    }
                }
                39 => self.sgr.fg = None,
                40..=47 => self.sgr.bg = Some((p - 40) as u8),
                48 => {
                    if i + 1 < collected.len() && collected[i + 1] == 5 {
                        if i + 2 < collected.len() {
                            self.sgr.bg = Some(collected[i + 2] as u8);
                            i += 2;
                        }
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

    /// Emit a full repaint to `out`. Dirty-cell diffing is a follow-up.
    pub fn render_full(&self, out: &mut impl Write) -> io::Result<()> {
        // Clear viewer screen + home cursor + reset SGR.
        out.write_all(b"\x1b[2J\x1b[H\x1b[0m")?;
        let mut last_sgr = SgrState::default();
        for r in 0..self.rows {
            out.write_all(format!("\x1b[{};1H", r + 1).as_bytes())?;
            for c in 0..self.cols {
                let cell = self.cells[self.idx(r, c)];
                if cell.sgr != last_sgr {
                    write_sgr(out, &cell.sgr)?;
                    last_sgr = cell.sgr;
                }
                let mut buf = [0u8; 4];
                out.write_all(cell.ch.encode_utf8(&mut buf).as_bytes())?;
            }
        }
        // Restore cursor to source's logical position.
        out.write_all(
            format!("\x1b[0m\x1b[{};{}H", self.cursor_row + 1, self.cursor_col + 1).as_bytes(),
        )?;
        out.flush()
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
            0x0A | 0x0B | 0x0C => {
                self.cursor_row = self.cursor_row.saturating_add(1).min(self.rows.saturating_sub(1));
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
}
