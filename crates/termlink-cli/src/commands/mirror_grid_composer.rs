//! Multi-session mirror composer (T-236).
//!
//! Composes N sessions matching a tag into a single TUI grid. Reuses the
//! per-session `Grid` primitive from `mirror_grid.rs`. Each session runs its
//! own vte parser inside a tokio task; the composite renderer paints all
//! panels into viewport sub-rectangles of the host terminal.

use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::sync::Mutex;

use super::mirror_grid::Grid;
use termlink_session::codec::FrameReader;
use termlink_session::data_server;
use termlink_session::manager;
use termlink_protocol::data::FrameType;
use crate::util::terminal_size;

/// Geometry of one panel within the composite terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PanelLayout {
    /// 0-based row of the panel's top edge (border row).
    pub row: u16,
    /// 0-based col of the panel's left edge.
    pub col: u16,
    /// Number of grid rows inside the panel (excludes 1-row label border).
    pub grid_rows: u16,
    /// Number of grid cols inside the panel (full width, no side border).
    pub grid_cols: u16,
}

/// Compute a balanced row × col arrangement for `n` panels.
///
/// Strategy: grid_cols = ceil(sqrt(n)), grid_rows = ceil(n / grid_cols).
/// Produces 1×1, 2×1, 2×2, 2×2, 3×2, 3×2, 3×3, 3×3, 3×3 for n=1..9.
pub(crate) fn compute_arrangement(n: usize) -> (usize, usize) {
    if n == 0 {
        return (0, 0);
    }
    let cols = (n as f64).sqrt().ceil() as usize;
    let rows = n.div_ceil(cols);
    (rows, cols)
}

/// Divide a terminal of (term_cols × term_rows) into layouts for `n` panels.
///
/// Each panel reserves 1 row at the top for a label/border. The remaining
/// rows are the grid area. Panels tile without overlap; the final column or
/// row may be slightly wider/taller to absorb rounding.
pub(crate) fn compute_layout(n: usize, term_cols: u16, term_rows: u16) -> Vec<PanelLayout> {
    if n == 0 || term_cols == 0 || term_rows == 0 {
        return Vec::new();
    }
    let (rows_n, cols_n) = compute_arrangement(n);
    let panel_w_base = term_cols / cols_n as u16;
    let panel_h_base = term_rows / rows_n as u16;
    let w_remainder = term_cols % cols_n as u16;
    let h_remainder = term_rows % rows_n as u16;

    let mut out = Vec::with_capacity(n);
    let mut row_cursor: u16 = 0;
    for r in 0..rows_n {
        let extra_h = if (r as u16) < h_remainder { 1 } else { 0 };
        let panel_h = panel_h_base + extra_h;
        let mut col_cursor: u16 = 0;
        for c in 0..cols_n {
            if out.len() >= n {
                break;
            }
            let extra_w = if (c as u16) < w_remainder { 1 } else { 0 };
            let panel_w = panel_w_base + extra_w;
            // Reserve top row for label; grid area is the rest.
            let grid_rows = panel_h.saturating_sub(1).max(1);
            let grid_cols = panel_w.max(1);
            out.push(PanelLayout {
                row: row_cursor,
                col: col_cursor,
                grid_rows,
                grid_cols,
            });
            col_cursor += panel_w;
        }
        row_cursor += panel_h;
    }
    out
}

/// Draw the label bar for one panel at its top row. Simple reverse-video
/// text showing `[index] name (id)` truncated to panel width.
fn draw_panel_label(
    out: &mut impl Write,
    layout: &PanelLayout,
    index: usize,
    label: &str,
) -> io::Result<()> {
    let text = format!(" [{}] {} ", index, label);
    let truncated: String = text.chars().take(layout.grid_cols as usize).collect();
    let pad = (layout.grid_cols as usize).saturating_sub(truncated.chars().count());
    out.write_all(format!("\x1b[{};{}H", layout.row + 1, layout.col + 1).as_bytes())?;
    out.write_all(b"\x1b[7m")?; // reverse video
    out.write_all(truncated.as_bytes())?;
    for _ in 0..pad {
        out.write_all(b" ")?;
    }
    out.write_all(b"\x1b[0m")?;
    Ok(())
}

/// One panel's mutable state. Wrapped in `Arc<Mutex<_>>` so the per-session
/// reader task can update it while the render tick thread snapshots it.
struct Panel {
    grid: Grid,
    parser: vte::Parser,
    label: String,
    closed: bool,
    dirty: bool,
}

impl Panel {
    fn new(cols: u16, rows: u16, label: String) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            parser: vte::Parser::new(),
            label,
            closed: false,
            dirty: true, // force initial paint
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.parser.advance(&mut self.grid, *b);
        }
        self.dirty = true;
    }
}

pub(crate) async fn cmd_mirror_tag(tag: &str) -> Result<()> {
    // 1. Discover sessions with matching tag.
    let all = manager::list_sessions(false).context("Failed to list sessions")?;
    let sessions: Vec<_> = all
        .into_iter()
        .filter(|s| s.tags.iter().any(|t| t == tag))
        .collect();
    if sessions.is_empty() {
        anyhow::bail!("No sessions with tag '{}'", tag);
    }

    // 2. Compute layout.
    let (term_cols, term_rows) = terminal_size();
    let layouts = compute_layout(sessions.len(), term_cols.max(1u16), term_rows.max(1u16));

    eprintln!(
        "Mirroring {} session(s) tagged '{}' in {}×{} terminal — Ctrl+C to stop.",
        sessions.len(),
        tag,
        term_cols,
        term_rows
    );

    // 3. Connect all data-plane sockets in parallel.
    let mut panels: Vec<Arc<Mutex<Panel>>> = Vec::with_capacity(sessions.len());
    let mut streams: Vec<tokio::net::UnixStream> = Vec::with_capacity(sessions.len());
    let mut ok_layouts: Vec<PanelLayout> = Vec::with_capacity(sessions.len());
    for (i, (reg, layout)) in sessions.iter().zip(layouts.iter()).enumerate() {
        let data_socket = data_server::data_socket_path(reg.socket_path());
        if !data_socket.exists() {
            eprintln!(
                "  [{}] {} — skipped (no data plane)",
                i, reg.display_name
            );
            continue;
        }
        match tokio::net::UnixStream::connect(&data_socket).await {
            Ok(stream) => {
                let label = format!("{} ({})", reg.display_name, reg.id);
                let panel = Panel::new(layout.grid_cols, layout.grid_rows, label);
                panels.push(Arc::new(Mutex::new(panel)));
                streams.push(stream);
                ok_layouts.push(*layout);
                eprintln!("  [{}] {} — connected", i, reg.display_name);
            }
            Err(e) => {
                eprintln!(
                    "  [{}] {} — skipped ({})",
                    i, reg.display_name, e
                );
            }
        }
    }
    if panels.is_empty() {
        anyhow::bail!("No data-plane connections established for tag '{}'", tag);
    }

    // 4. Paint initial frame: clear host, draw all labels.
    {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        out.write_all(b"\x1b[2J\x1b[H\x1b[0m")?;
        for (i, layout) in ok_layouts.iter().enumerate() {
            let panel = panels[i].lock().await;
            draw_panel_label(&mut out, layout, i, &panel.label)?;
        }
        out.flush()?;
    }

    // 5. Spawn one reader task per session; feed bytes into its panel.
    let mut readers = Vec::new();
    for (panel_arc, stream) in panels.iter().cloned().zip(streams.into_iter()) {
        readers.push(tokio::spawn(async move {
            let (read_half, _write_half) = tokio::io::split(stream);
            let mut buf_reader = tokio::io::BufReader::new(read_half);
            let mut frame_reader = FrameReader::new(&mut buf_reader);
            loop {
                match frame_reader.read_frame().await {
                    Ok(Some(frame)) => match frame.header.frame_type {
                        FrameType::Output => {
                            let mut p = panel_arc.lock().await;
                            p.feed(&frame.payload);
                        }
                        FrameType::Close => {
                            let mut p = panel_arc.lock().await;
                            p.closed = true;
                            p.dirty = true;
                            break;
                        }
                        _ => {}
                    },
                    Ok(None) | Err(_) => {
                        let mut p = panel_arc.lock().await;
                        p.closed = true;
                        p.dirty = true;
                        break;
                    }
                }
            }
        }));
    }

    // 6. Render tick loop: every ~33ms, paint dirty panels to stdout.
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("Failed to register SIGINT handler")?;
    let mut ticker = tokio::time::interval(Duration::from_millis(33));
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                let stdout = std::io::stdout();
                let mut out = stdout.lock();
                for (i, (panel_arc, layout)) in panels.iter().zip(ok_layouts.iter()).enumerate() {
                    let mut p = panel_arc.lock().await;
                    if !p.dirty {
                        continue;
                    }
                    // Grid area starts one row below the label.
                    let gr = layout.row + 1;
                    let gc = layout.col;
                    let _ = p.grid.render_diff_at(gr, gc, &mut out);
                    if p.closed {
                        let closed_label = format!("{} [CLOSED]", p.label);
                        let _ = draw_panel_label(&mut out, layout, i, &closed_label);
                    }
                    p.dirty = false;
                }
                let _ = out.flush();
            }
            _ = sigint.recv() => {
                eprintln!("\nMirror stopped.");
                break;
            }
        }

        // If all panels are closed, exit.
        let mut all_closed = true;
        for panel_arc in &panels {
            let p = panel_arc.lock().await;
            if !p.closed {
                all_closed = false;
                break;
            }
        }
        if all_closed {
            eprintln!("\nAll mirrored sessions closed.");
            break;
        }
    }

    for r in readers {
        r.abort();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrangement_1_is_1x1() {
        assert_eq!(compute_arrangement(1), (1, 1));
    }

    #[test]
    fn arrangement_4_is_2x2() {
        assert_eq!(compute_arrangement(4), (2, 2));
    }

    #[test]
    fn arrangement_6_is_2x3() {
        // 6 panels: sqrt(6) ≈ 2.45 → cols=3, rows=ceil(6/3)=2
        assert_eq!(compute_arrangement(6), (2, 3));
    }

    #[test]
    fn arrangement_9_is_3x3() {
        assert_eq!(compute_arrangement(9), (3, 3));
    }

    #[test]
    fn layout_single_panel_fills_terminal() {
        let layouts = compute_layout(1, 80, 24);
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].row, 0);
        assert_eq!(layouts[0].col, 0);
        assert_eq!(layouts[0].grid_cols, 80);
        assert_eq!(layouts[0].grid_rows, 23); // 24 - 1 label row
    }

    #[test]
    fn layout_divides_terminal_without_overlap() {
        // 4 panels in 80×24: 2×2 grid → each 40×12 (12 rows, 1 label + 11 grid).
        let layouts = compute_layout(4, 80, 24);
        assert_eq!(layouts.len(), 4);
        // Panel 0: top-left
        assert_eq!(layouts[0].row, 0);
        assert_eq!(layouts[0].col, 0);
        // Panel 1: top-right
        assert_eq!(layouts[1].row, 0);
        assert_eq!(layouts[1].col, 40);
        // Panel 2: bottom-left
        assert_eq!(layouts[2].row, 12);
        assert_eq!(layouts[2].col, 0);
        // Panel 3: bottom-right
        assert_eq!(layouts[3].row, 12);
        assert_eq!(layouts[3].col, 40);
        // No overlap: sum of widths equals term width per row.
        assert_eq!(layouts[0].grid_cols + layouts[1].grid_cols, 80);
        // Each panel has 1 label row → grid_rows = 12 - 1 = 11.
        assert_eq!(layouts[0].grid_rows, 11);
    }

    #[test]
    fn layout_absorbs_odd_dims_in_remainder() {
        // 2 panels in 81×25: cols=1 extra, rows=1 extra.
        // arrangement(2) = (1, 2) → 2 panels side by side.
        let layouts = compute_layout(2, 81, 25);
        assert_eq!(layouts.len(), 2);
        // First panel gets the +1 col remainder → 41 wide; second → 40.
        assert_eq!(layouts[0].grid_cols + layouts[1].grid_cols, 81);
        // Both panels span full terminal height.
        assert_eq!(layouts[0].grid_rows, 24); // 25 - 1 label
    }

    #[test]
    fn layout_zero_panels_is_empty() {
        assert_eq!(compute_layout(0, 80, 24).len(), 0);
    }
}
