---
id: T-2873
name: "termlink_remote_inject sends bare-string keys; broken against every hub"
description: >
  MCP termlink_remote_inject builds the command.inject keys array as bare strings (tools.rs:16062 - p.text.chars().map(|c| json!(c.to_string())), plus json!("Enter")), but the wire contract is KeyEntry with #[serde(tag="type", content="value")], so the hub rejects every call with -32602 'Invalid keys format: invalid type: string, expected adjacently tagged enum KeyEntry'. Reported by 100-Video-riper as suspected 0.9.0 vs 0.11.x hub skew on .122; disproved - reproduced byte-identical against the LOCAL 0.11.1716 hub, so the tool is broken against all hubs. The CLI sibling (remote.rs:1558-1562) and the local MCP sibling termlink_inject (tools.rs:12250) both wrap correctly - hardened in two places, this one never migrated. T-2747's parity census had already flagged termlink_remote_inject as unexamined (allowlist line 252, zero assertions in parity.rs).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-01T10:44:04Z
last_update: 2026-09-02T06:13:55Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2873: termlink_remote_inject sends bare-string keys; broken against every hub

## Context

`command.inject` takes `KeyEntry`, declared `#[serde(tag = "type", content = "value")]`
in `termlink-protocol` — an adjacently tagged enum, so every element must be
`{"type": ..., "value": ...}`. `termlink_remote_inject` built the array as **bare
strings, one per character**, so the hub rejected every call with `-32602 Invalid keys
format: invalid type: string "p", expected adjacently tagged enum KeyEntry` on the
FIRST character.

Reported by `100-Video-riper` as suspected `0.9.0` ↔ `0.11.x` wire skew against the
.122 hub. **Disproved.** The `0.9.0` reading came from session metadata, which is the
T-2744 defect — `termlink-session` stamped the workspace Cargo.toml constant into
every session ever created while the same binary reported `0.11.720` from
`--version`. `fleet doctor` puts .122 at **0.11.1411**. The failure then reproduced
byte-identically against the **local 0.11.1716 hub**, the newest in the fleet, which
kills the skew hypothesis outright: the payload was never valid, against any version.

Control, same hub and same session: the **CLI** path (`remote.rs` lines 1558-1562)
injected 20 bytes successfully; the **MCP** path failed. The local MCP sibling
`termlink_inject` (tools.rs:12250) also wraps correctly. Two surfaces got this right
and this one never migrated — the same "hardened in one place, sibling untouched"
shape as T-2667 / T-2673 / T-2687.

T-2747's parity census had already flagged `termlink_remote_inject` as *unexamined*
with zero assertions in `parity.rs`. The defect was sitting in exactly the gap the
census names, which is the clearest evidence so far that acknowledging 236 uncovered
tools buys something real.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The MCP tool sends a valid `KeyEntry` payload.** Construction extracted to `build_inject_keys(text, enter)` and fixed to emit `[{"type":"text","value":<text>}]` plus `{"type":"key","value":"Enter"}`. One Text entry for the whole string, matching the CLI sibling. Extracted rather than fixed in place because the old construction was inline in an async fn that needs a live hub to reach, which is exactly why no test could see it.
- [x] **The wire shape is proven, and proven able to fail.** Three tests decode the built array against the REAL `KeyEntry` type from `termlink-protocol`, not a hand-written JSON shape — a shape assertion would pass just as happily against the bare strings that caused the outage, since both are valid JSON (the PL-148 tautology). Load-bearing: restoring the exact pre-fix construction fails both positive tests (`MUTANT_RC=101`); restoring the fix returns green. A third test pins the regression directly — bare `["p"]` must NOT decode as `KeyEntry`; if it ever does, this diagnosis was wrong.
- [x] **The version-skew hypothesis is disproved, with evidence.** `fleet doctor` puts .122 at `0.11.1411`, not `0.9.0`; the `0.9.0` reading came from session metadata, which is the T-2744 defect (`termlink-session` stamped the Cargo.toml constant into every session ever created, while the same binary reported `0.11.720` from `--version`). The failure reproduced **byte-identically against the local `0.11.1716` hub** — the newest binary in the fleet — so no hub version is implicated. Control on the same hub and same session: the CLI path injected 20 bytes successfully while the MCP path failed.
- [x] **The parity census reason is corrected rather than cleared.** `termlink_remote_inject` stays in `.context/checks/mcp-parity-census-allowlist` because no `parity.rs` assertion exists yet and deleting the line would make the census claim coverage it does not have (T-2680). Its reason now records that the tool is no longer *unexamined* — it was examined and was broken — and that a full MCP-vs-CLI case is still owed under T-2748. Census re-runs clean: 260 tools, 24 asserted, 236 acknowledged, 0 unexamined.
- [x] **End-to-end through the MCP server.** PROVEN 2026-09-01, and proven on the RECEIVER rather than the sender. The blocker was never the fix — it was that the MCP server attached to this session is a long-lived process still running pre-fix code, which cannot be restarted from inside the session it serves. That does not require a restart to verify: a **fresh `termlink mcp serve` was spawned from the rebuilt binary** (`/root/.cargo/bin/termlink`, mtime 13:38, postdating the fix commit `93cb3d8d9` at 12:55) and driven over stdio with a real MCP handshake — `initialize` → `notifications/initialized` → `tools/call termlink_remote_inject`. Two runs: against a non-PTY tmux session the call returned `ok:false` with `"No PTY session"` — note the `-32602 Invalid keys format` is **gone**, so the `KeyEntry` payload now deserializes and reaches session resolution; against a PTY-backed session (`spawn --shell --backend background`) it returned `{"ok": true, "result": {"status": "injected", "bytes_len": 21}}`. **The hub saying `injected` is not proof** — that is this task's own finding — so `termlink output t2873pty` was read and shows `echo T2873_INJECT_OK` echoed and its output `T2873_INJECT_OK` on the following line. The command was injected AND executed. Server identified itself as `termlink-mcp 0.11.1766`.

### Human
- [ ] [RUBBER-STAMP] Restart the MCP server so the fixed `termlink_remote_inject` goes live.
  **Steps:**
  1. The fix is committed and `cargo build --release -p termlink-mcp` succeeds, but the MCP server serving this session is a long-lived process still running the pre-fix code — re-invoking the tool after the build still returns `-32602`, failing on the first character of the new text.
  2. Restart the MCP server (or the Claude Code session holding the `termlink` MCP connection) so it re-execs the rebuilt binary.
  3. Re-run an inject against a scratch session: spawn one with `termlink spawn --name probe -- bash -l`, then call `termlink_remote_inject` with `hub=192.168.10.107:9100`, `session=probe`, `text=echo OK`, `enter=true`.
  **Expected:** `{"ok": true, ...}` instead of `-32602 Invalid keys format`.
  **If not:** confirm the server actually re-execed — `/proc/<pid>/exe` showing `(deleted)` means it is still the old image (PL-209). This is the same shipped-vs-live trap as G-069 / preflight Check 5.


## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

grep -q "fn build_inject_keys" crates/termlink-mcp/src/tools.rs
! grep -q 'keys.push(serde_json::json!("Enter"))' crates/termlink-mcp/src/tools.rs
timeout 600 cargo test -p termlink-mcp inject_keys > /tmp/.t2873.txt 2>&1 && grep -q "build_inject_keys_round_trips_through_the_wire_type ... ok" /tmp/.t2873.txt && grep -q "build_inject_keys_appends_enter_as_a_key_entry ... ok" /tmp/.t2873.txt
timeout 600 cargo test -p termlink-mcp bare_string_keys > /tmp/.t2873b.txt 2>&1 && grep -q "bare_string_keys_are_rejected_by_the_wire_type ... ok" /tmp/.t2873b.txt
timeout 200 bash scripts/check-mcp-parity-census.sh > /tmp/.t2873c.txt 2>&1

## RCA

**Symptom:** every `termlink_remote_inject` call failed with `-32602 Invalid keys
format: invalid type: string "p", expected adjacently tagged enum KeyEntry`, against
every hub in the fleet, for the life of the tool. Reported externally as suspected
`0.9.0` ↔ `0.11.x` wire skew.

**Root cause:** `command.inject` takes `Vec<KeyEntry>`, and `KeyEntry` is declared
`#[serde(tag = "type", content = "value")]` — adjacently tagged, so each element must
be `{"type": ..., "value": ...}`. The MCP tool built the array as bare strings, one per
character. The payload was never valid against any hub version, which is why the skew
hypothesis died the moment it was tested against the newest binary in the fleet
(`0.11.1716`) rather than the oldest.

**Why structurally allowed:** three compounding gaps.
1. **No parity assertion.** `termlink_remote_inject` had zero cases in `parity.rs`.
   T-2747's census had already classified it as *unexamined* — the defect was sitting
   in precisely the gap the census names, which is the strongest evidence so far that
   enumerating 236 uncovered tools buys something real rather than being bookkeeping.
2. **Untestable construction.** The payload was built inline inside an async fn that
   needs a live hub to reach, so no unit test could observe it. Extracting
   `build_inject_keys` is what made the shape assertable at all.
3. **A shape test would not have caught it.** Bare strings are valid JSON, so any
   assertion written against a hand-authored JSON literal passes just as happily as
   against the correct payload — the PL-148 tautology. Only decoding against the REAL
   `KeyEntry` type from `termlink-protocol` can fail.

Two sibling surfaces — the CLI path (`remote.rs`) and the local MCP `termlink_inject`
(`tools.rs`) — already wrapped correctly. Same "hardened in one place, sibling never
migrated" shape as T-2667 / T-2673 / T-2687.

**A second, unrelated defect was found in this task's own file while verifying it.**
The task carried an ORPHANED block of template guidance: a stray `-->` at line 111 with
**no opening `<!--` anywhere above it**, and a line reading `` ## Verification` instead of
a Human AC here...`` sitting at **column 0** — a wrapped fragment of the template's
line 59, which is correctly indented in `.tasks/templates/default.md`. Because it was at
column 0 it read as a real `## Verification` heading, 25 lines above the genuine one.
`extract_verification_block` takes the FIRST match, so the P-011 gate for this task was
pointed at template prose (`1. Open https://example.com/dashboard in browser`) and its
five real verification commands were never reachable. Measured, then repaired: the
extractor now returns all five. Corpus scan: 23 of 2596 task files carry two
`## Verification` headings, but in 22 the first is the genuine one — **this file was the
only one actually mis-extracting**, so the incidence is 1/2596, not systemic.

**Prevention:** the wire-shape tests are the prevention for the reported defect — three
tests decode against the real `KeyEntry`, and the pre-fix construction fails them
(`MUTANT_RC=101`), so they can go red. The census line is deliberately kept rather than
deleted, because a full MCP-vs-CLI parity case is still owed under T-2748. For the
shadowed-heading defect the prevention is a separate guard and is filed as its own task
(one bug = one task): `check-verification-misfile.sh` is structurally blind to it, since
it looks for commands in the wrong SECTION and here the section itself is counterfeit.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Recommendation

**Recommendation:** GO — the reported defect is fixed and now proven live.

**Rationale:** The wire payload is correct, the fix is covered by three tests that decode
against the real `KeyEntry` type and are demonstrably able to fail, and the end-to-end
path has been exercised through a real MCP server built from the current source, asserted
on the receiving PTY rather than on the hub's own `injected` claim. Nothing about the
diagnosis rests on inference any more.

**Rationale for the remaining human step:** the only thing still outstanding is that the
MCP server **attached to this Claude Code session** continues to run pre-fix code, so the
tool remains broken *from this session* until that connection is restarted. That is a
property of the session, not of the code, and it cannot be actioned from inside the
session it serves.

**Evidence:**
- Fresh `termlink mcp serve` (`termlink-mcp 0.11.1766`) driven over stdio returned
  `{"ok": true, "result": {"status": "injected", "bytes_len": 21}}`.
- `termlink output t2873pty` shows `T2873_INJECT_OK` executed in the PTY — receiver-side
  confirmation, per this task's own finding that `injected` is a sender-side claim.
- The intermediate non-PTY run returned `"No PTY session"` rather than `-32602`, which
  independently confirms the payload now deserializes as `KeyEntry`.
- Installed binary mtime 13:38 postdates fix commit `93cb3d8d9` at 12:55.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-01T10:44:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2873-termlinkremoteinject-sends-bare-string-k.md
- **Context:** Initial task creation

### 2026-09-01T11:40Z — binary rebuilt, installed, stale MCP server stopped [claude-code]
- **Action:** Operator asked for the MCP server restart (the Human AC). Rebuilt
  `cargo build --release -p termlink-mcp -p termlink` (BUILD_RC=0, ~14 min under
  `-C lto -C codegen-units=1`), producing `termlink 0.11.1766`. Backed up the
  installed `termlink 0.11.1716` alongside itself as `termlink.pre-t2873.bak`,
  then installed via temp-file + `mv -f` rather than `install`: the destination
  was being executed by 12 live processes and a truncating open returns ETXTBSY.
  An atomic rename swaps the directory entry while running processes keep the
  old inode.
- **Config finding:** two MCP configs disagree about which binary serves this
  tool. `.mcp.json` says `command: "termlink"` (PATH); `.claude/settings.local.json`
  overrides it with `.termlink/bin/termlink`, which on this host is **0.9.13 from
  2026-03-27**, five months stale. Resolved empirically rather than by reading
  precedence rules: `/proc/<pid>/exe` of the live server showed
  `/root/.cargo/bin/termlink`, so the PATH entry is what actually runs and is what
  was upgraded. The stale local override is a live trap for anyone who trusts it —
  filed as a follow-up rather than edited here, since changing MCP wiring is not
  this task's scope.
- **Live A/B proof (the load-bearing evidence):** same hub, same session, same
  request, only the binary differs. Drove each binary as an MCP server over stdio
  (initialize → notifications/initialized → tools/call) against a scratch session
  `tl-5l34cgys` on `192.168.10.107:9100`:
    - OLD 0.11.1716 → `-32602 Invalid keys format: invalid type: string "O",
      expected adjacently tagged enum KeyEntry` — reproduces the peer's report
      byte-for-byte, failing on the first character.
    - NEW 0.11.1766 → `{"ok": true, "bytes": 15, "result": {"status": "injected"}}`.
  Hub-reported "injected" is a claim, not proof of arrival, so PTY ground truth was
  read separately: `termlink output tl-5l34cgys` shows `NEWBIN_OK_T2873` at the
  shell prompt. `OLDBIN` never appears. Scratch session and its tmux window reaped.
- **Server restart:** SIGTERM to pid 978915 (the `termlink mcp serve` child of this
  session's `claude`, started 2026-08-30, i.e. pre-fix). It exited; its
  `/proc/<pid>/exe` had already read `(deleted)`, which is the G-069 / preflight
  Check 5 signature — the running image was the replaced inode. Claude Code did
  **not** auto-respawn it: the 260 `mcp__termlink__*` tools went to disconnected.
  Reconnect (`/mcp`, or a new session) is the operator step that spawns a fresh
  server off the new binary; PATH now resolves to 0.11.1766, so the reconnect picks
  up the fix with no further action.
- **Not done, deliberately:** the Human AC box is left unchecked. The operator asked
  me to perform the restart, not to certify it; step 3 of that AC verifies through
  the *reconnected* server, which cannot exist until the reconnect happens. Ticking
  it now would assert a check nobody ran.
- **Fleet note:** 11 other `termlink mcp serve` processes belong to other sessions
  and all now show `exe ... (deleted)` — they hold the pre-fix inode and keep
  serving the broken `termlink_remote_inject` until each restarts. Expected, and
  the reason the binary swap was made non-disruptive rather than forced.

### 2026-09-02T08:20Z — human AC step 3 re-verified through a reconnected server [claude-code]
- **Action:** Ran the Human AC's own step 3 from a NEW session, through the MCP
  server this session is actually attached to — the surface the AC is about, and
  the one that could not be reached from the session that made the fix.
- **Server identity:** every `termlink mcp serve` started today (07:56, 07:57,
  08:11) has `/proc/<pid>/exe -> /root/.cargo/bin/termlink` with **no `(deleted)`
  suffix**, i.e. they hold the current inode, `termlink 0.11.1766` — the post-fix
  binary. The G-069 / preflight-Check-5 signature the AC's "If not" clause warns
  about is absent.
- **Result:** `termlink_remote_inject` against a scratch PTY session `t2873v` on
  `192.168.10.107:9100` returned `{"ok": true, "bytes": 26, "result": {"status":
  "injected", "bytes_len": 27}}`. The `-32602 Invalid keys format` is gone.
- **Receiver-side proof:** per this task's own finding that `injected` is a
  sender-side claim, `termlink_output t2873v` was read and shows the command
  echoed at the prompt AND its output on the following line:
  `echo T2873_LIVE_SESSION_OK` / `T2873_LIVE_SESSION_OK`. Injected AND executed.
- **Scratch session reaped** (`signal TERM` + `clean`, no stale sessions left).
- **Box deliberately left unchecked.** The evidence above is what the AC asks
  for, but checking a `### Human` AC is not an agent action under any delegation
  (CLAUDE.md § Agent/Human AC Split). This entry exists so the human can stamp it
  in one read rather than re-running the probe.
