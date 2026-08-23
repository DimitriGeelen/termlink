# Herdr adoption track 1 — terminal correctness (2026-08-15)

**Thesis of this pass:** herdr's ~142 open + ~230 closed issues are a corpus of
terminal-correctness failures found under heavy real use by a PTY implementation
younger than ours. Each is a candidate test case. This document maps every class
onto TermLink and states, with file:line, whether we handle it.

**READ-ONLY reconnaissance. No source was modified.**

**LICENCE POSITION: every recommendation below is category (a) — adopting an
IDEA (a bug class / a test case).** No herdr code is copied, quoted as
implementation, or transcribed. No attribution or NOTICE handling is triggered.
Issue *titles* are cited as evidence of a class existing, which is factual
reference, not derivative use. If any follow-up task proposes lifting herdr's
*implementation* of a fix, that is category (b) and must be re-flagged then.

---

## 0. Method and its limits

- Issue corpus: `gh api repos/herdrdev/herdr/issues` — open pages 1–3
  (~145 non-PR issues), closed pages 1–3 (~230 non-PR issues). Titles read for
  all; bodies not read. **A title is a weaker source than a body**; where a class
  turns on a detail I could not read from the title, it is marked as such.
- TermLink side: `crates/termlink-session/src/pty.rs` (720 lines, read in full),
  `crates/termlink-cli/src/commands/pty.rs` (1263 lines, read in full), plus
  targeted `grep` across `crates/` for each class.
- **Bash and grep were available this session** (unlike the T-2725 internal pass,
  which was `Read`-only). So absences below are repo-wide greps over
  `crates/ --include=*.rs`, not "zero in the files I opened". Each absence names
  the pattern searched.
- Two claims were checked **empirically** rather than by reading: the kernel's
  default winsize for `openpty(NULL)`, and the behaviour of `strip_ansi_codes`
  on non-alphabetic CSI finals (a faithful port of our own algorithm, run against
  concrete inputs). Both are shown inline.

**A "no handling found" below means: this grep, over these paths, returned
nothing.** Where I ran a grep whose zero result surprised me, I widened the
pattern and printed matching lines before concluding — the class-C and class-B
rows are the two where I did that.

---

## 1. The table

Severity is **for us**, judged by whether the defect can produce a *wrong answer*
(D2 — no silent failures) versus merely a *degraded display*.

| # | Issue class | herdr issues | TermLink? | Evidence | Sev |
|---|---|---|---|---|---|
| **A** | **Initial PTY window size** — panes stuck at a size floor when nothing is attached | #2828, #1709, #2625 | **NO** | `pty.rs:106–114` passes `std::ptr::null_mut()` as `openpty`'s `winp`. Grep for `TIOCSWINSZ\|winsize` over `crates/` returns exactly 3 product sites: `pty.rs:283/290` (the `resize()` method) and `util.rs:100` (the *client's* own tty). Grep for `.resize(` returns only `handler.rs:1214` (`command.resize` RPC) and `data_server.rs:152` (client Resize frame) — both caller-driven. `session.rs:279` spawns the PTY and never sizes it. Measured: `openpty` with NULL winp ⇒ **rows=0, cols=0** | **HIGH** |
| **B** | **Terminal left in a private mode after detach** (alt screen / mouse reporting / bracketed paste / kitty keyboard) | #2687, **#2581 (closed — upstream fixed it)**, #2310, #2309 | **NO** | `pty.rs:487–489` (attach) and `pty.rs:646–648` (stream) restore `termios` via `tcsetattr` and nothing else. Grep for `?1049\|?100[0-6]\|2004\|kitty` across `crates/` finds **no emission site at all** — the only `?1049` hits are the *detector* (`pty.rs:399–400`) and a mirror-grid test (`mirror_grid.rs:750/755`) | **HIGH** |
| **C** | **Nobody answers the child's terminal queries** (DSR `CSI 6n`, `CSI 14t/16t`, OSC 10/11/4) — child blocks, or the reply leaks to screen | #2472, #2471, #2786, #2440, #2823, #2609, #2630, #2717 | **NO responder** | Grep for `6n\|14t\|16t\|\\\\x1b\\[?[0-9]*n` and for OSC responders over `crates/`: nothing reads child output looking for a query. `handler.rs` treats PTY output as opaque bytes for scrollback only (`handler.rs:434–445`). The only ANSI *parser* in the tree is `mirror_grid` (`pty.rs:775–780`), which is a display consumer and writes nothing back | **HIGH** |
| **D** | **ANSI-strip mangles real text** — non-alphabetic CSI final bytes, DCS/APC/PM payloads | #1686, #2786, #2440 | **BROKEN** | Two near-identical impls: `crates/termlink-cli/src/util.rs:4–50` and `crates/termlink-session/src/ansi.rs:5–46`. Both terminate a CSI on `ch.is_ascii_alphabetic()`, so a `~`-final sequence runs past its end; both route DCS/APC into the `_ =>` "skip one char" arm, emitting the payload as text. Demonstrated below | **HIGH** |
| **E** | **Key-encoding table gaps** | #2536 (Ctrl+J), #1431 (Ctrl+[), #872 (Ctrl+6), #2770 (Ctrl+/), #2556 (Shift+Tab), #2705 (F1–F4), #1849 | **PARTIAL, fails loud** | `executor.rs:242–292`, read in full. Absent: **Ctrl+I, Ctrl+J, Ctrl+M** (table jumps H→K), **Ctrl+[ , Ctrl+] , Ctrl+^ , Ctrl+_**, **all F-keys**, **Shift+Tab (CSI Z)**, PageUp/PageDown, Insert, modified arrows. `resolve_key` returns `None` ⇒ `resolve_key_entry` errors `"Unknown key: {name}"` (`executor.rs:297`) ⇒ `resolve_keys` propagates. **Unknown keys are refused, not silently dropped** — D2-correct | **MED** |
| **F** | **Wide-char / VS-16 / CJK misalignment** | #2469, #1000, #1745 | **PARTIAL** | `mirror_grid.rs:3` imports `unicode_width`; `:185` computes width; `:19–20` models wide/continuation cells. But `:187` — *"Combining mark / zero-width — silently drop for v1"* — so U+FE0F (VS-16) is dropped, and `unicode-width = "0.1"` (`crates/termlink-cli/Cargo.toml:25`) predates emoji-presentation width. `➡️` is measured 1, drawn 2. **Mirror/display only** — the byte stream is untouched | **MED** |
| **G** | **Inherited session-id env var is trusted without validation** | #2012 | **NO validation** | `metadata.rs:538–540`: `session_hint.or(env_hint).or(name_hint)` — `$TERMLINK_SESSION_ID` is consumed and `find_session`'d with no check that the session owns the caller's process tree, and it **short-circuits ahead of** the T-1303 PID-walk (`metadata.rs:573+`). Same shape at `termlink-mcp/src/tools.rs:11830`. Seeded at `session.rs:277` and inherited by every descendant | **MED** |
| **H** | **Resize storm — no debounce; reflow loses scrollback** | #2665, #2625 | **NO debounce** | `pty.rs:907–915`: every `SIGWINCH` writes a Resize frame immediately. Same at `mirror_grid_composer.rs:338`. Grep for `debounce` over `crates/`: zero hits | **MED** |
| **I** | **UTF-8 split at a byte-offset cut ⇒ U+FFFD** | #1745 | **PRESENT** | `scrollback.rs:41–45` `last_n_bytes` cuts at an arbitrary offset; `scrollback.rs:22–38` `append` `drain(..overflow)`s at an arbitrary offset on ring overflow; `handler.rs:445` then does `String::from_utf8_lossy`. **Nuance: `last_n_lines` cuts on `\n`, so the default `lines` path is safe** — only `--bytes` and the ring-overflow path corrupt. Note `cmd_interact` polls with `bytes: 131072` (`commands/pty.rs:104–107`) | **MED** |
| **J** | **Alt-screen variants `?47` / `?1047` undetected** | (general) | **PARTIAL** | `pty.rs:399–400` matches only `\x1b[?1049h/l`. Grep `?47h\|?1047`: zero hits | **LOW-MED** |
| **K** | **Diff-renderer re-emits stale cells** | #2793, #2795, #1914 | **RISK, unproven** | `pty.rs:779` calls `grid.render_diff` per frame; grep for a periodic full-repaint escape hatch: none found. I did **not** read `mirror_grid.rs` end-to-end, so this is a *structural* risk note, not a confirmed defect | **LOW-MED** |
| **L** | **`SHELL` unset ⇒ `/bin/sh`** | #2641 | **SAME behaviour** | `pty.rs:88–90`. herdr filed this as a bug; we do the identical thing | **LOW** |
| **M** | **Zombie on drop** | #1228, #2749 | **RACY** | `pty.rs:456–467`: `Drop` issues `SIGKILL` then *immediately* `waitpid(..., WNOHANG)`. The child has almost certainly not been reaped in that instant, so `WNOHANG` returns 0 and the zombie survives until process exit | **LOW** |
| **N** | **CPU live-lock / busy-spin** | #2592 | **ALREADY GUARDED** | `scripts/check-busy-spin.sh` (T-2672) + the T-2670/2671/2673 migrations. No action | — |
| **O** | **Exit codes and signals** | herdr has *neither* (synthesis §3) | **WE HAVE BOTH** | `pty.rs:308–327` (`waitpid` + `WIFEXITED`/`WIFSIGNALED`, incl. `128+sig`), `pty.rs:447–453` (`kill`). This is our architectural advantage, not a gap | — |

### Class D, demonstrated

A faithful port of `util.rs:4–50` / `ansi.rs:5–46`, run on concrete inputs:

```
CSI ~ final (Delete key echo)   '\x1b[3~hello world'           -> 'ello world'
bracketed paste markers         '\x1b[200~pasted text\x1b[201~ tail'
                                                               -> 'asted textail'
DCS body (ESC P ... ST)         '\x1bP1$r0m\x1b\\visible'      -> '1$r0mvisible'
APC (ESC _ ... ST)              '\x1b_Gf=100;payload\x1b\\visible'
                                                               -> 'Gf=100;payloadvisible'
CSI intermediate final          '\x1b[?1;2$ytail'              -> 'tail'
normal SGR (control)            '\x1b[0mplain'                 -> 'plain'
```

This is not cosmetic. `--strip-ansi` is sold as "give me the clean text", and it
**deletes real characters** (`h` from `hello`, `p` from `pasted`) and **emits
escape-sequence payloads as if they were output**. Both directions are wrong
answers from a surface whose contract is fidelity.

### Why A, B, C are HIGH and cluster

They share one root: **TermLink models a PTY that nobody is watching.** That is
the right model for the charter (a programmatically-consumed command) — and it
is exactly why nothing sizes the terminal (A), nothing tears down the modes the
child turned on (B), and nothing answers the questions the child asks (C).
herdr's whole design assumes an attached human, so it hit these as *rendering*
bugs; we get them as *the child misbehaves and no one can tell*.

Concretely: `termlink spawn --shell` then `termlink interact <s> 'vim …'` runs
vim at 0×0, which asks for the cursor position, which nothing answers.

Also worth stating plainly, because it is the honest read of A: this is not a
defect herdr taught us about so much as one *their bug report gave us the
vocabulary to look for*. #2828 is about an 80×24 **floor**; we have no floor at
all. Their bug is our worse bug.

---

## 2. Ranked regression tests to add

Ordered by (probability the defect is live) × (cost of the wrong answer). Each is
a test we write ourselves against our own code — category (a) throughout.

1. **`pty_spawn_has_nonzero_winsize`** — spawn a `PtySession`, `TIOCGWINSZ` the
   master, assert `rows > 0 && cols > 0`. **Fails today** (measured 0×0). Pair
   with an end-to-end assert that `register --shell` produces a session where
   `stty size` in the child reports a sane default. *D2 Reliability, D3
   Usability.*
2. **`strip_ansi_preserves_text_after_tilde_final`** — assert
   `strip("\x1b[3~hello") == "hello"`, and the bracketed-paste and DCS/APC cases
   above. **Fails today.** Add to *both* `util.rs` and `ansi.rs` — the duplicate
   impls are their own finding, and a shared test is what will force the merge.
   *D2.*
3. **`detach_restores_private_modes`** — drive a child into alt screen + mouse
   reporting + bracketed paste, detach, assert the reset sequences were written
   to the client's stdout. **Fails today.** herdr closed #2581 for this; the
   class is proven to bite in the field. *D2, D3.*
4. **`output_bytes_never_emits_replacement_char`** — write a buffer of multi-byte
   UTF-8, query `--bytes N` at an N that lands mid-char, assert no U+FFFD.
   Second case: overflow the ring mid-char and assert the same. **Fails today**
   for both. A `char_boundary_floor` already exists at `commands/pty.rs:999` —
   the fix is to apply the same discipline at `scrollback.rs:41`. *D2.*
5. **`interact_survives_a_query_only_child`** — a child that emits `\x1b[6n` and
   waits. Assert `interact` returns a *timeout naming the cause* rather than
   hanging to the deadline with an empty diff. Even without building a responder,
   the failure must be legible. *D2.*
6. **`resolve_key_covers_the_control_range`** — assert Ctrl+I/J/M, Ctrl+[/]/^/_,
   F1–F12, Shift+Tab, PageUp/PageDown. **Fails today.** Also pin the *good*
   behaviour we already have: `inject --key F1` must **error**, never silently
   inject nothing (`executor.rs:297`) — that assertion is load-bearing and
   currently untested. *D2, D3.*
7. **`whoami_rejects_a_foreign_inherited_session_id`** — set
   `TERMLINK_SESSION_ID` to a live session that does **not** own the caller's
   ancestor chain; assert `whoami` does not report it as the caller.
   **Fails today** (`metadata.rs:538`). Mirror on the MCP surface
   (`tools.rs:11830`) — parity, per the T-2687 `parity_topics` lesson. *D2.*
8. **`alt_screen_detects_47_and_1047`** — the older variants. Extend the existing
   split-read tests at `pty.rs:621–669`, which are already the right shape. *D2.*
9. **`drop_reaps_child_without_zombie`** — assert no zombie remains after
   `PtySession` drop (`pty.rs:456–467` needs a blocking wait, or a short retry).
   *D1 Antifragility.*
10. **`mirror_grid_width_of_vs16_emoji`** — pin current behaviour, then decide.
    This one is **display-only** and the honest move may be to accept it and say
    so, rather than pull in a `unicode-width` major bump for a mirror. *D3.*

Tests 1–4 are where I would spend the first day. All four are live defects, all
four produce a **wrong answer** rather than a bad-looking one, and none needs a
herdr dependency, a herdr line of code, or a licence conversation.

---

## 3. What this pass did NOT establish

- **Issue bodies were not read** — only titles. The mapping from a herdr title to
  a class is my inference. #2828 ("80x24 floor") and #2581 ("leaks CSI sequences
  into parent shell") are the two I would most want to read in full before
  quoting them in a task, because both are load-bearing for a HIGH row.
- **Class K (stale cells) is a structural risk note, not a confirmed defect.** I
  did not read `mirror_grid.rs` end-to-end.
- **I did not run our test suite**, so "fails today" for tests 1, 2, 4, 6, 7 is
  derived from reading the code and from the two empirical checks — not from a
  red test. Test 3's "fails today" is the strongest of these (there is simply no
  emission site anywhere in the tree); test 1's is measured.
- **Windows/ConPTY classes were excluded as out-of-scope**, not as handled.
  A large share of herdr's traffic (#2810, #2536, #2726, #1183, …) is ConPTY.
  TermLink is `libc::openpty` + `fork` (`pty.rs:106–120`) and does not target
  Windows — but note that makes those issues *irrelevant*, not *passed*.
- **macOS** was not separately assessed. Given T-2692's macOS CI is still
  non-blocking and T-2693 exists precisely because `/proc` assumptions leaked in,
  classes A/B/M deserve a macOS-specific read that this pass did not do.
