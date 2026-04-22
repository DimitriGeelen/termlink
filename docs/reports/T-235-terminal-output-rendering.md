# T-235: Research Clean Terminal Output Rendering for Mirror Mode

**Status:** Recommendation drafted — GO on vte+grid approach
**Related:** T-234 (mirror command, shipped), T-236 (multi-session mirror, follow-on), T-243 (script error yielding)

## Problem

`termlink mirror` (T-234) streams raw PTY bytes from source to viewer via the data plane (`cmd_mirror` at `crates/termlink-cli/src/commands/pty.rs:595`). The read path is a byte-faithful pass-through: every `FrameType::Output` payload is written to viewer stdout unmodified. This works for scrollback-style append-only output (shells running `ls`, `cat`, etc.) but degrades for:

1. **Full-screen TUIs** (vim, htop, less, ncurses apps). The source terminal uses CSI moves to redraw arbitrary cells; the viewer's terminal re-interprets those moves against its own cursor state and geometry. Visual artefacts: double-draw, scrollback pollution, cursor left in arbitrary position.
2. **Geometry divergence.** Source `cols×rows` may differ from viewer's; cursor-addressable redraws land on the wrong cells.
3. **State divergence.** Source's alternate-screen / scrollback / cursor-visibility modes leak into the viewer and persist after disconnect.
4. **Multi-session composition (T-236 GO path).** If mirror is to show N PTYs in a grid, raw pass-through has no grid model to compose against.

Parallel surface: `termlink interact --strip-ansi` and `termlink output --strip-ansi` (pty.rs:17, pty.rs:173) already use a hand-rolled filter in `util::strip_ansi_codes` (crates/termlink-cli/src/util.rs:4). This is a lossy CSI/OSC remover — drops colour, boldness, everything, not only motion. Good for log-grepping output, wrong for live mirror fidelity.

## Research Areas

### Approaches considered

| # | Approach | Fidelity | Complexity | T-236 fit |
|---|----------|----------|------------|-----------|
| A | Byte pass-through (status quo) | Full for scrollback apps; broken for TUIs/geometry-drift | Zero | None — no grid |
| B | Strip-ANSI filter (existing `strip_ansi_codes`) | Text-only, drops colour | Trivial (already shipped) | None — lossy |
| C | vte tokenise + in-process grid | Full for cursor-addressable output; lossless colour/attr | Medium | Natural — grid is the shared abstraction |
| D | `alacritty_terminal` full emulator | Maximum (alt-screen, scrollback, modes, images) | Heavy (transitive alacritty deps) | Good — already grid-based |
| E | `termwiz` (wezterm) | Between C and D | Medium-heavy | Good |

### Rust crate landscape

**`vte`** (Alacritty's parser crate, `vte ^0.13`).
- Scope: tokenizer only. Implements the Paul Williams ANSI/VT state machine. You supply a `Perform` trait impl with callbacks (`print`, `execute`, `csi_dispatch`, `osc_dispatch`, etc.).
- Dependencies: essentially none (arrayvec, utf8parse). Small binary footprint.
- Trade-off: You write the grid model yourself. That's 200-400 lines to cover cursor motion, EL/ED erase, scroll region, SGR state — but no more.

**`alacritty_terminal`** (the Alacritty terminal state machine as a library, `alacritty_terminal ^0.23`).
- Scope: full grid, cursor, scrollback, modes, selections. Feeds a renderer via `Grid<Cell>`.
- Dependencies: heavy (serde, log, bitflags, vte, unicode-width, transitive on alacritty types). Adds ~1.5-2 MB to release binary.
- Trade-off: fidelity is maximal but over-scoped for a read-only CLI mirror; pulls in rendering-oriented types we don't need.

**`termwiz`** (wezterm's terminal lib).
- Scope: cell grid + parser + renderer helpers. `termwiz::surface::Surface` is a grid you can write escape sequences into and then diff for rendering.
- Dependencies: medium (anyhow, bitflags, regex, terminfo, unicode-segmentation). Integration is more opinionated than vte.
- Trade-off: nice ergonomics, but the renderer wants its own output layer which overlaps with our stdout write.

**`ansi-parser`** (`^0.9`).
- Scope: iterator-based classification of input into `AnsiSequence` variants.
- Dependencies: nom.
- Trade-off: higher level than vte (named enum variants vs byte-level callbacks) but not a grid. Suitable for filters, not emulation.

## Technical Constraints

- **No TTY on viewer.** `termlink mirror` may run in a plain pipe (CI log collection, `termlink mirror foo | less`). A grid-based renderer must emit bytes that work in a dumb terminal, and must not assume `isatty(stdout)`.
- **Variable geometry.** Viewer's terminal may be smaller or larger than source. Mirror must either resize-to-viewer (reflow) or letterbox.
- **Unicode width.** CJK, emoji, combining marks — the grid needs `unicode-width` for cell counting. Both alacritty_terminal and termwiz handle this; vte does not (you wire it).
- **Binary size.** termlink aims for small release binaries (Homebrew + musl variants per T-1134/1135). Avoid alacritty_terminal's dep tree unless fidelity demands it.
- **No new unsafe.** Stay in safe Rust.
- **Streaming, not buffered.** `mirror_loop` reads per-frame; a grid must update incrementally and emit incremental diffs to stdout, not redraw-from-scratch on each frame.

## Scope Fence

**IN scope:**
- Recommendation on how to render cursor-addressable PTY output cleanly in `cmd_mirror`.
- Viability of the chosen approach as foundation for T-236 (multi-session grid composition).
- Trade-off analysis (fidelity / binary size / maintenance burden).

**OUT of scope:**
- Implementing the chosen approach (a separate build task after GO).
- Changes to `cmd_interact` / `cmd_output` `strip_ansi` path — that's a text filter for post-hoc log consumption, orthogonal to mirror fidelity.
- Alternate-screen buffering across disconnects (T-243 territory if relevant).
- Images / sixel / kitty graphics protocols.

## Assumptions

1. **A-1: Mirror's primary use case is TUI apps, not scrollback logs.** If users only mirror `cat`/`ls`/`grep` output, approach A (pass-through) is already fine.
   _Test_: survey episodic memory / usage tracing. What commands have been mirrored in practice? Scan `.context/episodic/` for mirror invocations.
2. **A-2: Grid emulation at 200-400 LoC covers 90% of real-world PTYs.** The long tail (DCS passthrough for graphics, DEC private modes for mouse) rarely matters for dev/ops sessions.
   _Test_: build the minimum grid against vte; run against vim + htop + less + bash; measure coverage by counting unhandled CSI dispatches.
3. **A-3: vte's `Perform` trait is stable enough to hand-write a small grid.** The API hasn't broken since 0.10.
   _Test_: read vte's CHANGELOG / crate docs.
4. **A-4: T-236 (multi-session grid) is worth the grid abstraction now.** If T-236 is unlikely to ship, C is over-engineering and B+A suffice.
   _Test_: re-read T-236's inception state.

## Go/No-Go Criteria

**GO if:**
- vte + minimal grid ≤400 LoC covers bash, vim, htop, less cleanly in a spike.
- Binary size delta for vte is <100 KB.
- T-236 is a credible follow-on (not abandoned).

**NO-GO if:**
- A spike shows the grid needs >1000 LoC to cover real-world TUIs (then adopt alacritty_terminal's grid instead, pay the binary cost).
- vte's trait callback surface has hidden gotchas (encoding edge cases, DCS handling) that blow the LoC budget.
- T-236 is abandoned AND A-1 holds (mirror is mostly used for scrollback) — then ship with just A+B and close T-235.

## Recommendation

**GO on C — `vte` tokeniser + in-process grid model.**

Rationale (anchored to the directives):

- **Antifragility.** Unknown CSI sequences become observable events (unhandled callbacks we can count) rather than visual artefacts we can't attribute.
- **Reliability.** A grid model makes the viewer's output a deterministic function of the source's frame stream; no more "double-draw depending on viewer geometry."
- **Usability.** TUI mirror becomes genuinely readable; multi-session grid (T-236) becomes tractable because the shared abstraction exists.
- **Portability.** vte is ~120 KB, safe Rust, works on musl. No alacritty/wezterm transitive bloat.

Rejected: D (alacritty_terminal) — right tool for a renderer, wrong tool for a CLI mirror. Carries types (Selection, ColorIndex, etc.) and deps (transitive on alacritty crates) we do not need.

Rejected: B-only — already implemented; its limits are the reason T-235 exists.

Rejected: E (termwiz) — more ergonomic than vte but couples to a render layer that overlaps with our stdout-owning `mirror_loop`.

## Follow-on Build Task Scope (for post-GO)

1. Add `vte = "0.13"` + `unicode-width` to `termlink-cli/Cargo.toml`.
2. Create `crates/termlink-cli/src/commands/mirror_grid.rs` with:
   - `struct Grid { cols, rows, cells: Vec<Cell>, cursor, scroll_region, sgr_state }`
   - `impl vte::Perform for Grid { ... }` handling: `print`, `execute` (CR/LF/BS/BEL), `csi_dispatch` for CUP/CUU/CUD/CUF/CUB/EL/ED/SGR/DECSET(25/1049)/DECRST.
   - `fn render_diff(&mut self, out: &mut impl Write) -> io::Result<()>` that emits only changed cells since last render (dirty-region compression).
3. Wire into `mirror_loop`: feed each `FrameType::Output` payload through `vte::Parser::advance(&mut grid, byte)`, then render-diff to stdout.
4. Gate with `--raw` flag for users who want byte pass-through (status quo).
5. Tests in `crates/termlink-cli/tests/mirror_grid.rs` with golden-input→golden-output pairs for: vim opening a file, htop 1-second refresh, `ls --color`, `less` paging through a file.
6. Benchmark: emit 1 MB of vim session traffic, assert render latency <16 ms per frame.

Complementary validation via termlink itself:
- Spawn a sender session: `termlink spawn vim-demo -- vim /etc/passwd`
- From another shell: `termlink mirror vim-demo` (old path) vs `termlink mirror vim-demo --grid` (new path)
- Compare: does the old path leave artefacts when viewer cols < source cols? Does the new path render cleanly?

## Dialogue Log

_(No human dialogue yet — research artifact built from code read + crate knowledge. Awaiting human REVIEW of recommendation.)_

## Decisions

### 2026-04-22 — approach selection
- **Chose:** vte tokeniser + in-process minimal grid (approach C)
- **Why:** smallest dep footprint that still gives cursor-addressable fidelity; natural grid for T-236 composition; observable unknown-CSI surface supports antifragility.
- **Rejected:** alacritty_terminal (dep weight), termwiz (render-layer overlap), ansi-parser (not a grid), byte pass-through (status quo — broken for TUIs), strip-ansi-only (lossy).
