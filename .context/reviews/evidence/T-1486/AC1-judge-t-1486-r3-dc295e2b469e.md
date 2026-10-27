# T-1486 Human AC#1 — watch view steady (no flicker / no row jitter)

Reviewer dispatch: judge-t-1486-r3-dc295e2b469e — revision 340365b03bbf6613fffffcb0e90bebe60958c697

## What I ran
I ran `target/release/termlink agent presence --watch --watch-interval 3 --window-secs 86400` from /opt/termlink.
It ran under a real pty (python pty.fork), so stdout was a TTY. It ran for about 10.5s. Then I sent `\x03` (Ctrl-C) through the tty line discipline.
I captured the raw bytes with per-read timestamps.

## Findings against "Expected"
- **The screen clears between iterations: MET.** Every tick starts with `ESC[2J ESC[H`. There were 4 frames at t=0.01, 3.03, 6.05 and 9.08s, so the 3s cadence holds.
- **The columns are aligned: MET.** The header is `{:<18} {:>14} {:>8}  TOP_PROJECT` and the data rows use the same widths.
  In the captured frames, LAST_SEEN right-edges at col 33 and POSTS at col 42 in both the header and the rows (`d1993c2c3ec44c94          39m ago       30  010-termlink`).
  The row order and widths were identical across all 4 frames. There is no row jitter.
- **The header timestamp updates every tick: MET.** 14:56:55Z → 14:56:58Z → 14:57:01Z → 14:57:04Z. LAST_SEEN also ticked 39m→40m in frame 4 with the alignment intact.
- **It exits on Ctrl-C cleanly: MET.** waitpid returned exit status 0 at once, and no signal death.
  The code (crates/termlink-cli/src/commands/agent.rs ~1900-1960) never touches termios, raw mode, the alt screen or cursor visibility. Only `ESC[2J ESC[H` is emitted, so it leaves no odd terminal state.

## Residual flicker (why not green)
The loop runs `print!("\x1b[2J\x1b[H")` and then the header `println!` *before* the hub fetch (agent.rs ~1904-1945).
The screen is therefore blank apart from the header while the fetch runs, and the table body only arrives afterwards.
The pty timing shows two writes per tick: 101 bytes (clear and header), then 199 bytes (table) **20-30 ms later** on every tick (0.01/0.03, 3.03/3.05, 6.05/6.08, 9.08/9.10).
On the local hub that is a 1-2 frame blank, a faint blink each tick. On a remote `--hub`, or a slow or erroring hub, the blank lasts for the whole fetch latency and becomes a visible flash.
I judged this from the byte stream and timing. I did not watch it with human eyes.

## Suggested concrete improvement
1. Fetch first, then render the whole frame (header, table and footer) into a `String`/`Vec<u8>`.
2. Emit `ESC[H` + frame + `ESC[J` (home, overwrite, clear-to-end) in one `write_all` and `flush`. This avoids the full `ESC[2J` blank. Optionally emit `ESC[K` at each line end.
   This is the same pattern mirror_grid already uses ("second render must not full-clear", mirror_grid.rs:995).
