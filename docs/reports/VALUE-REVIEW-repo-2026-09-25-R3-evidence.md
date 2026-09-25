# Value review — repo — 2026-09-25 — Round 3 evidence (T-3093 R3S1)

Scope: whole repo (/opt/termlink). Round 3 of 4 in the [Review, Audit, procAsFit] × 4
sequence. Fed by R2S3's handback (procAsFit round 2: closed T-3133, bisected T-3130
negative, unblocked arc-011 via T-3134/T-3135).

**Delta round, not a from-scratch review** — the tenth value-review pass over this repo,
the third in this orchestrated sequence. Same convention as R1S1/R2S1: reconfirm the
yardstick/data-map (unchanged), then spend the round closing out the prior round's own
explicit carry-forwards and reporting genuinely new evidence, rather than re-walking the
full Phase 2 inventory (last done 2026-09-19, still current per R1S1/R2S1's own
reconfirmation).

## 1. Orientation

- Read the run record + R2S3's fed handback (inlined in dispatch) + R1S1/R2S1's own
  reports and evidence files before gathering anything new, per the delta-round
  convention both prior rounds established.
- `git log --oneline -20` confirmed no commits landed between R2S3's close (`fdcedbe02`)
  and this step's dispatch other than the orchestrator's own bookkeeping commit
  (`bd1981222`) — nothing to reconcile against a moving target.
- Session context at start: 182,437 tokens (~22%) via `checkpoint.sh status` (the
  per-session reader — `.budget-status` is still documented as unsafe for a concurrently
  dispatched worker to trust, per T-3127/CLAUDE.md's own carried warning; not re-verified
  this round since R2S1/R2S2 already did so twice and it is now structurally documented).

## 2. Data availability map (delta only)

| Source | R2S1 status | R3S1 status | Evidence |
|---|---|---|---|
| Invocation-audit sink (`invocation-audit.jsonl`) | EXISTS, but STALLED at 4 records, ~7h after going live | **STILL EXACTLY 4 records, same 2 timestamps, now ~9.5h+ since first observed** — AND root-caused this round (see §3 F1) | `wc -l /var/lib/termlink/invocation-audit.jsonl` = 4; `tail` shows the identical `1790289251771`..`1790289258892` window R1S1 first cited |
| R2-F4 (anomalous no-tty `mcp serve` process, pid 1895596) | INVESTIGATE, single data point, ~32h uptime | **RESOLVED — see §3 F2.** Same pid, now 121,146s (~33.6h) elapsed, confirmed to be a live Claude Code **Desktop app** session's MCP child (parent pid 1895454 is a live, non-orphaned `claude --output-format stream-json ...` process with `ccd_*`/computer-use tool grants — architecturally has no controlling tty by design) | `ps -o pid,ppid,lstart,etimes,cmd -p 1895596`; `ps -o pid,ppid,tty,cmd -p 1895454`; `/proc/1895596/status` shows `PPid: 1895454`, `State: S` (not orphaned/reparented to init) |
| R1-F1/F1b (4 unused Cargo deps) | still present, unexecuted (expected — no approval step ran) | **still present, unexecuted — now the 3rd consecutive round reporting this unchanged**, ~10h and 2 full audit-remediation cycles since first proposed with zero human engagement on the [ASK] | `crates/termlink-{protocol,test-utils}/Cargo.toml` re-read, lines unchanged |
| G-087-class `.budget-status` concurrency hazard (R2-F2) | ADD (SURFACE), freshly filed this round | **CLOSED — the CLAUDE.md paragraph exists** (line ~2025-2040, cites T-3127/T-3128 by number, matches R2-F2's exact proposed wording in substance) | `grep -n "T-3127" CLAUDE.md` → 2 hits; `T-3127` task exists in `.tasks/active/`; `T-3128` (the CTL-003 audit-side sibling) also exists, `status: started-work` |
| arc-011 slice drift (S1/S2/S12) | not yet a factor — arc-011 still blocked at R2S1 | **RESOLVED by R2S3** — re-pointed to T-3134 (S2) and T-3135 (S1+S12), `check-arc-slice-drift.sh` reads 0 firing (not re-run this round; R2S3's own verification stands, no state changed since) | R2S3 handback; `.context/arcs/arc-011.yaml` |
| arc-009 backlog | 7/18 (R1S1 baseline) + 2 (R1S3) | **26 of ~49 tagged tasks now `work-completed`**, 3 `started-work` (T-3006, T-2984, T-3010 — T-3010 intentionally left open per R1S3), 20 `captured` including **T-3032 and T-3033 — the exact CLI/daemon invocation-audit follow-ups this round's F1 independently rediscovered** | `grep -rl "arc-009" .tasks/{active,completed} | xargs status read` — full listing in §3 F1 |

## 3. New findings this round (verified, with citation)

### F1 — Invocation-audit sink stall is ROOT-CAUSED this round: the CLI surface was never wired, and a task already exists for it

R1S1 found the sink live with 4 records. R2S1 found it stalled at the same 4 records ~7h
later and left the REPAIR-vs-WIRE question open, deferring a code read as "out of scope
for Phase 3." This round did that code read.

`crates/termlink-mcp/src/server.rs:44` calls `termlink_hub::invocation_audit::record(SURFACE_MCP,
&request.name)` unconditionally in the `call_tool` trait method — this is real, correct,
and (per its own doc comment) infallible-by-design. It fires for every genuine MCP
protocol tool call (`mcp__termlink__*` invocations).

`crates/termlink-hub/src/invocation_audit.rs:68` also defines `SURFACE_CLI: &str = "cli"` —
but `grep -rn "SURFACE_CLI\|invocation_audit::record" crates/ --include=*.rs` (excluding
the module's own test block) returns **exactly one production call site, and it is the MCP
one**. `SURFACE_CLI` is referenced nowhere except the test suite that exercises the
constant in isolation. There is no CLI-verb choke point recording anything.

Every unit of work all three orchestrated `claude -p` workers in this sequence have done
so far (R1S1-R2S3, ~10 hours) has gone through the `termlink`/`fw` **CLI**, never a literal
`mcp__termlink__*` tool call — confirmed by re-reading every prior handback's commands (all
`fw`, `git`, `bash scripts/*.sh`, direct `termlink <verb>`). That is exactly the surface
this instrument does not record. The 4 static records are not evidence of low usage; they
are the ceiling of what this instrument CAN see given its current wiring, and this round's
own three-round, ~10-hour, multi-cycle measurement window is itself the direct empirical
demonstration.

**This is not a new gap — it is already filed.** `.tasks/active/T-3032-cli-verb-invocation-telemetry--surfacecl.md`
(created 2026-09-21, 4 days before this sequence started, `status: captured`,
`horizon: next`, tagged `arc:arc-009`) states the identical root cause verbatim: *"T-2996
shipped invocation_audit with SURFACE_CLI defined and zero call sites: the MCP tool
surface is instrumented, the CLI verb surface is not... Until this lands,
invocation-usage.sh can only answer 'not observed on the MCP surface', never 'unused' —
and C-45's usage question spans both surfaces."* It carries a BVP proposal (D1=4, D2=2,
D3=3, D4=2) and a cost estimate (tier=2, effort=8, blast_radius unmeasured), never
executed. A sibling, `.tasks/active/T-3033-session-daemon-kvsession-invocation-blin.md`,
covers the same gap for the `kv.*`/`session.*` daemon RPC surfaces (C-30/C-31).

Non-use diagnosis for the SINK ITSELF (not a downstream consumer — there still is no
consumer, this is Phase-1 "does the writer even fire for the traffic that matters"
territory): **reading B, NEVER WIRED, with intent evidence directly in the module's own
doc comment** ("Recorded explicitly so a future CLI/session-daemon instrument can share
the sink without the reader having to guess from the name" — `invocation_audit.rs:65-66`).
The module's own author anticipated exactly this gap and left the constant as a marker for
it. This is not ambiguous.

**Consequence, stated plainly:** the module's own doc comment names its purpose as "the
evidence base for DELETING tools" and calls UNDERCOUNT "the dangerous failure direction."
As currently wired, any DELETE verdict drawn from this sink today would be judging 260 MCP
tools' usage against a data source that has never once seen the surface every orchestrated
agent session in this repo actually uses. That is a live risk to every future review round
that reaches for this sink as DELETE evidence, not just an abstract gap.

### F2 — R2-F4 (anomalous no-tty `mcp serve` process) resolves to KEEP: a live Desktop-app session, not an orphan

R2-F1 (R1S1) first noted ~30 long-lived `mcp serve` processes without classifying them;
R2S1 (R2-F3) cross-referenced 30/31 against `pts`/`who` and closed those as legitimate;
R2-F4 held back the 31st (pid 1895596, `tty=?`, ~32h uptime) as INVESTIGATE, naming the
resolving test explicitly: "a repeat check next round... growing/persistent uptime with
still no tty would strengthen an orphan reading; disappearing would resolve it as a
transient headless worker."

This round: `ps -o pid,ppid,lstart,etimes,cmd -p 1895596` — same pid, `etimes=121146`
(~33.65h, consistent growth from R2S1's ~32h reading given the ~2h36m gap between R2S1 and
this step's dispatch), started `Thu Sep 24 00:13:15 2026`. Its own predicted test
(persistent + growing + still no tty) fired — but the follow-up check it also predicted
(parent-process archaeology) resolves it the OTHER way from what a naive read of "growing
+ no tty" would suggest:

`ps -o pid,ppid,tty,cmd -p 1895454` (its parent) shows a live, running process:
`/home/dimitri-mint-dev/.config/Claude/claude-code/2.1.270/claude --output-format
stream-json ... --allowedTools mcp__computer-use,mcp__ccd_session__spawn_task,...` — the
`ccd_*`/`computer-use`/window/sidebar tool grants are the signature of a **Claude Code
Desktop app session**, not a terminal or an orchestrated `claude -p` worker. `/proc/1895596/status`
confirms `PPid: 1895454` and `State: S` (sleeping, not reparented to pid 1) — i.e. NOT an
orphan; its parent is alive and still supervising it. A Desktop-app-launched session has no
controlling pty by construction (it isn't attached to any terminal), which fully explains
`tty=?` without requiring a leak.

Non-use diagnosis: not applicable — this was never a non-use question, it was an identity
question (leaked vs legitimate), and it is now resolved with a positive mechanism (a live,
non-orphaned parent) rather than by absence of contrary evidence.

### F3 — R1-F1/F1b (4 unused Cargo deps): unchanged a third time, and the [ASK] silence itself is now the more interesting data point

Re-read `crates/termlink-protocol/Cargo.toml:11-12` (`bytes`, `ulid`) and
`crates/termlink-test-utils/Cargo.toml:9-10` (`serde_json`, `termlink-protocol`) — byte-
identical to R1S1's original citation. No regression, no silent pickup by either
audit-remediation cycle (R1S2 filed 33 tasks from `fw audit`/`fw doctor` findings, none of
them this one; R2S2 filed 6 more from a fresh audit run, same). This is now the cleanest,
smallest, most fully-evidenced DELETE candidate this entire sequence has produced (all 7
DELETE CHECKS passed since round 1), sitting unactioned through 3 review rounds, 2
audit-remediation cycles, and 3 procAsFit rounds — roughly 10 hours and 7 of this
sequence's 9 completed steps — purely because Phase 6 requires human per-item approval and
no human has engaged with either this specific [ASK] or the broader review-queue backlog
R1S1 separately flagged (144 items, still 144 per R2S1, not re-checked this round to avoid
burning budget on an expected-unchanged number).

## 4. Contradictions

None new. The version-skew triplet noted in R1/R2 evidence files was not re-checked this
round (no code changed that would move it; re-verifying an unchanged number a third time
is exactly the "polluting what is measured" / budget-discipline the ground rules warn
against).

## 5. Not reviewed

- Full Phase 2 inventory (unchanged since 2026-09-19; three rounds now of delta-only
  confirmation, consistent with R1S1/R2S1's own precedent that this repo's inventory is
  stable at the current activity level).
- JS/Python dead-code tooling (knip/vulture/jscpd) — still ABSENT, unchanged since R1S1,
  not re-flagged as new evidence.
- T-2486's pre-decision `revisit_at` anomaly — checked its `## Decisions` section this
  round (still template-only, no recorded rationale for the early `revisit_at`) but not
  chased further; single data point, unchanged priority from R2S1.
- A fresh baseline (`cargo test --workspace` / guard layer) — no code changed since R1S1
  that would invalidate the earlier decision to skip it.
- `fw review-queue` re-run to check whether the 144-item backlog has moved — skipped this
  round to conserve budget; R1S1's number is carried forward as last-known, not re-verified
  as unchanged (distinguish from F3 above, which IS a re-verified fact).
