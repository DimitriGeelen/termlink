# TermLink stabilisation backlog — ranked (2026-08-15)

Derived from the audit passes 1–4 and the herdr evaluation. Every item below was
observed this session with a cited source; nothing here is speculative. Ranked by
**risk × certainty**, not by effort.

**Next session: work this list top-down, and fire the herdr workers in parallel
(they cost no parent context).** Briefs are in
`.context/upstream/herdr-adoption-dispatch-briefs.md` — operator approved 8
termlink agents on 2026-08-15; the `check-agent-dispatch` gate is cleared.

---

## P0 — silently wrong right now

### 1. Fleet runs a known silent-data-loss bug
Three hubs serve **0.11.720**; the T-2533 fix landed in **0.11.871**
(`ac859d321`, 2026-08-08). Until upgraded, `count-1` fallback silently
under- or over-reports message counts. Floors were raised in T-2720, so the
fleet-binary canary now names them daily — that is correct behaviour, not noise.
**Action:** install ≥0.11.871 and restart **through the systemd unit** (G-070 —
a detached restart produces the ghost-process class). Operator foothold needed.

### 2. Two cursor stores that never reconcile (T-2719, still open)
`agent inbox` reads local `~/.termlink/cursors.json`; `agent unread` /
`channel unread` read the hub-side ack receipt. Measured divergence on
`agent-chat-arc`: **388 vs 30**, a 12.9× gap.
**Critical:** upgrading to 0.11.871 alone makes the printed number *worse* — a
correct `latest` of 11867 minus a stale local cursor of 1611 = 10256, against a
true 30. Do **not** treat the binary upgrade as the fix. Reconcile the stores.

### 3. CTL-010 reports the Tier-0 log empty while it holds 15 authorizations
(U-009, high) `.context/bypass-log.yaml` has two writers with incompatible
schemas; CTL-010 counts `grep -c "commit:"` → 0 against 15 real entries
(force pushes, `pkill -9`). Both arms of its test are `pass`, so once the file
exists **no content can make it fire**. Vendored — the fix is upstream, but the
false PASS is live here today.

## P1 — a guard that would go dark, not red

### 4. `session-selftest.sh:271` hardcodes `--backend tmux`
If tmux is ever removed or swapped, the **prover breaks before the product
does** and the T-2557 session-control canary goes dark rather than red. This is
the exact failure class the guard layer exists to catch, and it would be
self-inflicted. Fix the hardcode independently of any herdr work — it is wrong
today regardless.

### 5. CLI/MCP parity drift is structural, not incidental
`parity_topics` failed silently from **2026-08-12** until T-2686 wired
`cargo test` into CI; T-2687 fixed that instance. The *mechanism* — parity
maintained by hand — is untouched, so the next divergence is a matter of time.
**This is herdr worker 3's brief** (their CLI and socket API are the same
surface by construction). Highest-leverage item in the whole backlog: a
structural fix retires the class.

### 6. Five upstream tasks in the G-066 finalization-bypass class
T-2713, T-2714, T-2715, T-2717 sit `started-work` with **zero ticked ACs**
while their deliverables (U-002…U-005) exist and are substantial. T-2711 was
repaired this session as the worked example: verify each AC against the record,
add what is genuinely missing, route the `framework:pickup` filing to a `### Human`
AC with a `## Recommendation` block, then complete to partial-complete
(`owner: human`). Repeat for the other four.
**Do not bulk-tick.** Each AC makes a specific claim; verify it or add the
content that makes it true.

### 7. T-2711 verification fails 1 of 3
`out=$(bash .agentic-framework/agents/context/revisit-due-scan.sh 2>&1); echo "$out" | grep -q "tasks dir not found"`
fails under P-011 though it passes when run directly — most likely a cwd/env
difference in the gate's execution context. **Do not `--skip-verification`.**
Diagnose it; it may be a P-011 defect worth its own upstream record.

## P2 — accumulated, low-risk, human-gated

- **Arc `arc-substrate-fitness` stale** — sovereign. Needs T-2250's revisit
  (22 days overdue) or `fw arc approve-driver … --i-am-human`.
- **T-1898 revisit** — 41 days overdue.
- **Learnings promotion** — 6 candidates triaged in the remediation plan;
  PL-168 (D1), PL-213 (D2), PL-206 (D2) recommended; PL-209/195/172 not.
- **Nine upstream records** await the operator's `framework:pickup` filing call.
- **Fabric: 1 carded-but-unwatched file** — the 34-slash-command decision
  (register all with real edges, or drop the singleton `capture.md` card).

## Known-unclearable — do not spend effort here

**Fabric "195/346 cards have no edges" cannot be cleared.** The threshold is
`-gt 10` **absolute** (`audit.sh:1541`), requiring 185 of 195 cards to gain
edges on a graph where 71 depend on a compiled binary a file→file model cannot
name. Filed as U-006 (high). Any future pass reporting 0 WARN should be checked
for what it faked or narrowed — the audit itself suggests touching a timestamp
as a mitigation, which is fabrication.

---

## Standing methodology note

Three findings this session were wrong because one measurement was generalised
past what it measured — twice in opposite directions on the *same* number, and
one had already been filed outbound before it was checked. The rule that fell
out, and that every worker brief now carries:

> A count of zero deserves more suspicion than a count of a few, and any count
> headed somewhere outbound deserves its lines printed and read first.

Recorded as PL-341's second-order lesson.
