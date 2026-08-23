# TermLink internal session-control / tmux surface (2026-08-15)

Companion to `.context/upstream/herdr-evaluation-recon-2026-08-15.md`. Scope:
INTERNAL only — what would have to change to replace or supplement the session
backend, and whether there is a clean seam. No herdr research here.

**READ-ONLY reconnaissance. No code was modified.**

---

## TOOLING CONSTRAINT — read this before trusting any number below

This investigation ran with **both Bash and Agent dispatch blocked**:

- `Bash` — blocked by the `budget-gate` PreToolUse hook (session at ~106% of
  window; only commit/push/handover permitted).
- `Agent` — blocked by `check-agent-dispatch` (dispatch limit 2, already
  exhausted before this task began).

So there was **no `grep`, no `glob`, no `fw fabric`, no `wc -l`**. Everything
below comes from the `Read` tool on paths inferred from module manifests
(`lib.rs`, `commands/mod.rs`) — which is a sound way to *find* code but a
**useless way to prove absence**.

Consequences, stated plainly rather than buried:

- I **cannot** give an exhaustive census of `tmux` occurrences across `crates/`
  and `scripts/`, and I do not claim one. Deliverable #2 as briefed
  ("count call sites") is **NOT satisfied**.
- Every count below is qualified with exactly which file I read to get it.
- Deliverable #4 (fabric `impact`/`deps`) is **NOT satisfied** — `fw` could not
  be run. Dependency claims are from reading source imports only.
- **Any "zero" below means "zero in the files I opened", never "zero in the
  repo."** The brief warned about exactly this failure mode; the honest version
  is that a couple of the most interesting findings are *bounded absences*, and
  a grep-enabled follow-up should confirm them.

What follows is high-confidence on the **positive** findings (the seam exists,
here is its shape, here are the line numbers) and explicitly low-confidence on
the **negative** ones (no tmux output-parsing anywhere).

---

## 1. Headline: the premise of the question is wrong

**tmux is not TermLink's session backend.** TermLink owns its PTYs natively.

`crates/termlink-session/src/pty.rs` creates terminals with
`libc::openpty()` → `libc::fork()` → `execvp()` (lines 103–120), wraps the
master fd in a tokio `AsyncFd`, and keeps a `ScrollbackBuffer`. `PtySession`
(pty.rs:52–68) holds `master_read`, `master_write_fd`, `child_pid`,
`scrollback`, `alternate_screen`, `last_mode`.

`crates/termlink-session/src/lib.rs` lists 30 modules. There is **no `tmux`
module and no `backend` module**. The terminal primitives are `pty`,
`scrollback`, `executor`, `manager`, `handler`, `server`, `data_server`.

tmux appears in the product only as **one of three ways to launch a detached
process** which then runs `termlink register`. Once registered, that session is
driven entirely over TermLink's own JSON-RPC on a per-session Unix socket.

So the correct mental model is:

```
                    ┌─ Terminal.app (osascript)  ┐
termlink spawn ─────┼─ tmux new-session -d       ┼──> sh -c "termlink register --shell ..."
  (launcher only)   └─ setsid sh -c              ┘                    │
                                                                      v
                                              TermLink OWNS the PTY (libc::openpty)
                                                                      │
     inject / output / resize / exec / signal / attach / stream / mirror
                                                                      │
                            JSON-RPC over <session>.sock  +  binary data plane
```

The launcher is discarded after it has done its one job. Nothing reads tmux
state back (see §3).

---

## 2. The backend abstraction — it already exists, and it is clean

**Enum:** `SpawnBackend { Auto, Terminal, Tmux, Background }`, declared in
`crates/termlink-cli/src/cli.rs` (imported at
`crates/termlink-cli/src/commands/execution.rs:12`).

**Single dispatch site** — `execution.rs:326–331`:

```rust
let resolved = resolve_spawn_backend(&backend);
let spawn_result = match resolved {
    SpawnBackend::Terminal   => spawn_via_terminal(&session_name, &shell_cmd),
    SpawnBackend::Tmux       => spawn_via_tmux(&session_name, &shell_cmd),
    SpawnBackend::Background => spawn_via_background(&session_name, &shell_cmd),
    SpawnBackend::Auto       => unreachable!("resolve_spawn_backend always resolves Auto"),
};
```

That is the **entire** polymorphism. Three sibling free functions, one uniform
signature `(session_name: &str, shell_cmd: &str) -> Result<()>`:

| Function | Location | Eyeballed LOC | Mechanism |
|---|---|---|---|
| `spawn_via_terminal` | execution.rs:507–526 | 20 | `osascript -e 'tell application "Terminal" … do script'` |
| `spawn_via_tmux` | execution.rs:528–539 | **12** | `tmux new-session -d -s tl-<name> <shell_cmd>` |
| `spawn_via_background` | execution.rs:541–561 | 21 | `setsid sh -c` with `sh -c` fallback |

**Auto-detection** — `resolve_spawn_backend`, execution.rs:473–505: on macOS
probe `pgrep -x WindowServer` → `Terminal`; else probe `tmux -V` → `Tmux`;
else `Background`.

**The payload is backend-agnostic.** `build_spawn_shell_cmd`
(execution.rs:401–471) produces a plain POSIX shell string —
`export TERMLINK_RUNTIME_DIR=…; /path/to/termlink register --name X --shell …`
— with no backend awareness whatsoever. All three backends receive the same
string.

**Verdict: the seam is clean.** One enum, one match, three ~15-LOC functions, a
shared string payload, and a `Display` impl. Adding a variant is a mechanical
change.

### Total tmux-specific product code I actually read: ~22 lines

`spawn_via_tmux` (12) + the `tmux -V` probe inside `resolve_spawn_backend` (~10).
Plus one naming convention: the tmux session is named `tl-<session_name>`
(execution.rs:529).

**Bounded-absence caveat:** that is 22 lines *in `execution.rs`*. I did not read
`crates/termlink-mcp` (which exposes a `termlink_spawn` MCP tool and may
duplicate or re-implement this), nor `crates/termlink-hub`, nor the bulk of
`scripts/`. A grep-enabled pass must confirm before "~22 LOC" is quoted as a
repo-wide figure.

---

## 3. The property that makes the swap cheap: no read-back

I read `spawn_via_tmux` in full. It runs one command and checks
`status.success()`. It does **not**:

- parse `tmux list-sessions -F …`
- scrape `tmux capture-pane`
- drive `tmux send-keys`
- consult `tmux has-session`
- store the tmux session name anywhere for later lookup

The `tl-<name>` string is **write-only**. TermLink's identity for a session is
its own `SessionId` plus the `display_name` in the registration JSON under
`sessions_dir` (`manager.rs:35–115`), and readiness is established by **polling
TermLink's own registry**, not by asking the launcher:

```rust
// execution.rs:355–387  (--wait)
loop {
    if let Ok(reg) = manager::find_session(&session_name) { /* ready */ }
    if start.elapsed() > timeout { /* bail */ }
    tokio::time::sleep(Duration::from_millis(250)).await;
}
```

Output-format parsing is the brittlest possible coupling to a multiplexer, and
in the launcher path it appears **absent**. That single fact is most of why the
seam is cheap.

*(Confidence: high for `execution.rs`, which I read end-to-end around the spawn
path. Not established repo-wide — see the tooling constraint.)*

---

## 4. Where the actual mass is — `pty.rs` is NOT reachable through tmux

I read `crates/termlink-cli/src/commands/pty.rs` **in full (1263 lines)**. It
implements `interact`, `output`, `inject`, `resize`, `attach`, `stream`,
`mirror`, plus pure helpers and ~40 unit tests.

**It contains zero occurrences of the string `tmux`.** This is a whole-file
read, so unlike the claims in §3 this absence is genuinely established for this
file.

Every operation is TermLink-native RPC to the session's own socket:

| Verb | Transport | Sink |
|---|---|---|
| `inject` | `command.inject` RPC | write to PTY master fd |
| `output` | `query.output` RPC | `ScrollbackBuffer` |
| `resize` | `command.resize` RPC | `ioctl TIOCSWINSZ` |
| `attach` | `query.output` poll + `command.inject` | raw-mode local termios |
| `stream` | binary data plane, `FrameReader`/`FrameWriter` | `FrameType::{Output,Input,Resize,Close}` |
| `mirror` | data plane + `vte` parser → `Grid` | read-only repaint |

That is the real "control terminal sessions" implementation, and **the spawn
backend is not in its path at all**.

### exec has two exit-code paths — and only one is faithful

This is the most decision-relevant technical detail found.

**Path A — `termlink exec` / `termlink run` (faithful).**
`executor::execute` runs the command as a **direct child process**, and the exit
code is the real `status.code()`. `session-selftest.sh:198` cites
`executor.rs:229` as `status.code().unwrap_or(-1)`. Note this path needs **no
PTY at all**: selftest STAGE 1 spawns `-- sleep 30`, which reports `pty: null`
(selftest.sh:240–242), and `exec` still works against it.

**Path B — `termlink interact` (lossy, sentinel-scraped).**
`cmd_interact` (pty.rs:12–178) injects
`"{command}; echo \"{marker} exit=$?\""` into the PTY, then **polls scrollback
and string-parses the exit code back out** (`has_marker`, `parse_exit_code`,
`extract_clean_output`, pty.rs:925–969).

Path B is what any multiplexer-mediated backend forces you into, because a
multiplexer gives you a terminal, not a process handle. **This is the hidden
cost of moving `exec` onto a session-server backend: exit-code fidelity
degrades from `status.code()` to scraping `exit=$?` out of a terminal buffer.**

That fidelity is explicitly guarded. `session-selftest.sh` STAGE 2b runs
`sh -c 'exit 7'` and asserts `exit_code == 7` exactly (lines 200–228), with a
comment stating why `ok` is deliberately not the discriminator. A backend swap
that routed `exec` through a PTY sentinel would have to keep passing that stage.

---

## 5. Capability checklist

Split in two, because the brief's question ("replace the tmux backend") and the
strategic question ("adopt herdr") land on **different** checklists.

### 5a. To add a new SPAWN LAUNCHER (the actual tmux seam) — 5 requirements

| # | Requirement | Evidence | Grade |
|---|---|---|---|
| L1 | Launch a **detached** process from one POSIX **shell-command string** | all three backends take `shell_cmd: &str`; execution.rs:401–471 builds it | HARD |
| L2 | Survive the launching `termlink spawn` process exiting | `spawn_via_background` uses `setsid`; tmux `-d`; Terminal.app is a separate app | HARD |
| L3 | Propagate env vars into the child | `export TERMLINK_RUNTIME_DIR=…` + `--env` pairs, execution.rs:435–444 | HARD |
| L4 | Report **launch** success/failure synchronously | `status.success()` in all three; error surfaces at execution.rs:332–342 | HARD |
| L5 | Run **headless** (no GUI, no display server) | `Background` is the `Auto` fallback when neither WindowServer nor tmux is present, execution.rs:473–505 | HARD |

**Explicitly NOT required of a launcher** — each is served by TermLink itself:
readiness signalling (registry polling, execution.rs:355–387); a stable
session handle (write-only `tl-<name>`); output capture; input injection;
resize; signals; liveness; exit codes.

### 5b. To replace the SESSION BACKEND itself (what herdr would be) — 12 requirements

Derived from the verbs and the prover, not speculation.

| # | Requirement | Demanded by | Grade |
|---|---|---|---|
| S1 | `exec` returns the child's **real** exit code, exact non-zero fidelity | selftest STAGE 2b asserts `== 7`; executor.rs:229 | HARD |
| S2 | `exec` signals **truncation** distinctly from success | selftest STAGE 2 requires `truncated != true`; T-2537/T-2529 | HARD |
| S3 | `exec` works on a session with **no PTY** | STAGE 1 spawns `sleep 30`, `pty: null`, exec still passes | HARD |
| S4 | `inject` distinguishes **delivered** from **resolved-but-nowhere** | `inject_status_is_injected`, pty.rs:273–275; T-2697/T-2580 | HARD |
| S5 | Injected keys are **interpreted by the shell**, not merely echoed | selftest STAGE 3b's quote-stripping sentinel, lines 246–252 | HARD |
| S6 | `inject` works on a **detached** session (no attached client) | every selftest stage runs detached | HARD |
| S7 | Byte-accurate **scrollback** with a cumulative `total_buffered` counter | `compute_output_delta`, pty.rs:976–990 | HARD |
| S8 | ANSI-strippable text output | `--strip-ansi`, `query.output` `strip_ansi` param | HARD |
| S9 | **Real-time byte stream** with Output/Input/Resize/Close framing | `stream_loop`, pty.rs:817–920; data plane | HARD |
| S10 | PTY **resize** (cols/rows), plus SIGWINCH propagation | `cmd_resize` pty.rs:380–448; `stream_loop` SIGWINCH | HARD |
| S11 | **Signal delivery** to the child process (`signal <s> TERM`) | selftest STAGE 4 cleanup, lines 323–333 | HARD |
| S12 | Terminal **mode introspection** (canonical/echo/raw/alt-screen) | `TerminalMode`, pty.rs:11–22; `termlink pty-mode` | SOFT — one consumer |

Plus two structural ones that are easy to miss:

- **S13 (HARD):** per-session addressability from a *separate process*.
  TermLink's registry is a directory of JSON + a Unix socket per session
  (`manager.rs:35–115`); any replacement must let an unrelated `termlink`
  invocation find and drive an existing session.
- **S14 (HARD):** survive **SSH disconnect / lid close**. Not proven by the
  selftest (which is same-host and short-lived) but assumed everywhere by
  `tl-claude.sh` persistent mode and the be-reachable push-waker.

---

## 6. Blast radius

**Fabric not consulted** — `fw fabric impact/deps` could not be run (Bash
blocked). The brief asked for it; it is missing. Below is source-read only.

**Swapping/adding a spawn launcher (§5a): contained.**
- `crates/termlink-cli/src/cli.rs` — one enum variant + `Display` arm.
- `crates/termlink-cli/src/commands/execution.rs` — one match arm, one ~15-LOC
  function, optionally one `resolve_spawn_backend` probe branch.
- Tests: `resolve_backend_*_passthrough`, `resolve_backend_auto_returns_valid`,
  `spawn_backend_display` (execution.rs:680–724) — all mechanical.
- **UNVERIFIED:** whether `crates/termlink-mcp`'s `termlink_spawn` tool
  duplicates the enum. Must be checked before estimating.

**Replacing the session backend (§5b): that is the product.**
Reading `lib.rs`, the implicated modules are `pty`, `scrollback`, `executor`,
`manager`, `handler`, `server`, `data_server`, `registration`, `liveness`,
`lifecycle`, `codec` — plus every RPC method, the whole of
`commands/pty.rs` (1263 lines), the MCP mirror, and the CLI verbs.

**Guards that must be re-proven either way:**
- `scripts/session-selftest.sh` (T-2485) — 7 stages: SPAWN, EXEC,
  EXEC_EXITCODE, PTY_SPAWN, OUTPUT, INJECT, CLEANUP.
- `scripts/check-session-control-freshness.sh` (T-2557) — daily canary wrapping
  that prover; exit 1 = verb-4 regression, exit 2 = tooling.

**Trap:** `session-selftest.sh:271` hardcodes `--backend tmux`:

```bash
"$TERMLINK" spawn --name "$PTY_SESSION" --shell --backend tmux \
    --wait --wait-timeout 10 …
```

If tmux is removed or deprecated, **the prover breaks before the product does**,
and per T-2684's contract a guard that cannot run is `ERROR`, not `PASS` — the
canary would go dark exactly when scrutiny is needed. Any change touching the
tmux variant must migrate this line in the same commit.

---

## 7. Charter verb 4: how tmux-specific is it?

Charter wording could not be re-read (`docs/CHARTER.md` not opened — no grep to
locate the verb-4 paragraph). Using the four capabilities enumerated in
`session-selftest.sh:230–237`, which decomposes the charter noun: peers can
**stream output, inject keystrokes, exec, and doorbell-wake** sessions.

| Capability | tmux-specific? |
|---|---|
| stream output | **No** — data plane + `ScrollbackBuffer` |
| inject keystrokes | **No** — `command.inject` → PTY master fd |
| exec | **No** — direct child process, `executor::execute` |
| doorbell-wake | **No** — be-reachable push-waker against a termlink-owned PTY |
| *get a process running in the first place* | **Yes — and only this** |

**Verb 4 is ~95% backend-agnostic.** tmux is one of three interchangeable
bootstrap mechanisms, chosen by autodetect, discarded immediately, never read
back. On a headless Linux host with no tmux installed, `Auto` resolves to
`Background` (`setsid`) and **every** charter-verb-4 capability still works.

Corollary for the strategic question in the companion doc: adopting herdr would
**not** be replacing tmux — tmux is ~22 lines of launcher. It would be replacing
`termlink-session`'s PTY/RPC/scrollback/data-plane layer, i.e. the substance of
the founding verb.

---

## 8. Effort estimate

| Scope | Estimate | Confidence |
|---|---|---|
| Add a 4th `SpawnBackend` variant (herdr/other as a **launcher**) | **~0.5 day** — enum variant, `Display` arm, one match arm, one ~15-LOC fn, 3 mechanical tests | High — I read the whole seam |
| Add it *and* make it the `Auto` default on some hosts | +0.5 day — one probe branch + revisiting the T-2693 platform-lock allowlist | Medium |
| Replace `termlink-session`'s PTY layer with an external session server | **Weeks–months, and a charter decision** — reimplements S1–S14, rewrites `commands/pty.rs` (1263 LOC), the RPC surface, the MCP mirror, the data plane | Medium — scope read, not costed |

---

## 9. What would make a swap dangerous

1. **Exit-code fidelity regression (highest risk).** `exec` today is a direct
   child with `status.code()`. A session-server backend pushes you onto the
   `cmd_interact` shape — inject `; echo "MARKER exit=$?"`, then string-scrape it
   back (pty.rs:925–947). That is strictly weaker: it breaks on binary output,
   on commands that never return to a prompt, and on prompt-echo collisions.
   selftest STAGE 2b (`exit 7`) exists to catch precisely this.

2. **The prover hardcodes `--backend tmux`** (session-selftest.sh:271). Break it
   and the T-2557 canary goes dark rather than red.

3. **`inject`'s injected/resolved distinction is hard-won.** T-2694 found
   `inject` reporting `{"ok":true,"bytes_injected":18}` for a total no-op;
   T-2697 fixed it; three load-bearing tests pin it (pty.rs:1019–1052). A new
   backend must reproduce the *distinction*, not just the happy path.

4. **Directive #4 / platform-lock.** `spawn_via_background`'s `setsid` calls are
   among the 8 sites acknowledged in `.context/checks/platform-lock-allowlist`,
   each carrying a stated non-Linux degradation story. A new backend needs that
   story written before it merges, or `check-platform-lock.sh` fires.

5. **Licence contagion reaches further than the seam suggests.** The launcher
   seam is a subprocess boundary — cheap and licence-safe. The §5b replacement
   is *linking*, which is exactly where the unresolved AGPL-vs-Apache question in
   the companion doc bites. **Cheap seam ≠ cheap adoption**: they are different
   integrations with different legal shapes.

6. **A second answer to verbs 1 and 3.** Per the companion doc, an agent-session
   server brings its own presence and blocked/idle model, competing with
   `agent-presence`/`find-idle` and lease-based claims. Not a code risk; a
   charter risk — and the T-2468 review already flagged breadth accretion.

---

## 10. Follow-ups this investigation could not do

Ordered by how much they would change the conclusions:

1. **`grep -rn tmux crates/ scripts/ docs/`** and eyeball every hit — the only
   way to convert §2's "~22 LOC" and §3's "no read-back" from *bounded* to
   *established*. If a `tmux list-sessions -F` or `capture-pane` exists anywhere
   I did not read, the "clean seam" verdict weakens.
2. **Read `crates/termlink-mcp`'s `termlink_spawn`** — does it duplicate
   `SpawnBackend`, or delegate to the CLI? Changes the §8 estimate directly.
3. **`fw fabric deps/impact`** on `termlink-session/src/pty.rs` and
   `commands/pty.rs` — and if no cards exist, that is itself a finding worth a
   gap entry (the founding verb's subsystem unmapped in the fabric).
4. **Read `docs/CHARTER.md` verb 4 verbatim** — §7 currently substitutes the
   selftest's decomposition for the charter's own words.
5. **Confirm S14** (survive SSH disconnect) is actually tested anywhere. I found
   no test for it; it may be assumed rather than proven.
