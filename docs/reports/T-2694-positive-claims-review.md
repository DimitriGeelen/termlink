# T-2694 — TermLink purpose review #5: are the charter's POSITIVE claims true and proven?

**Type:** inception (exploration → go/no-go)
**Status:** exploration complete, recommendation GO on the in-authority subset
**Predecessors:** T-2468, T-2678, T-2683, T-2690

---

## The question

Four reviews have run, and between them they now cover all four Constitutional
Directives:

| Review | Question | Axis |
|---|---|---|
| T-2468 | Does the product match the charter? | Directives #1 / #2 |
| T-2678 | Does anything *enforce* the charter's **non-goals**? | #1 / #2 |
| T-2683 | Does anything *execute* the enforcement? | #1 / #2 |
| T-2690 | Are the Usability and Portability promises verified? | #3 / #4 |

T-2678 built a claim-vs-guard matrix for the charter's **five non-goals** and found
them guarded 2/5. Nobody has ever built the same matrix for the other half of the
charter — the section headed *"What TermLink is (the load-bearing nouns)"*:

> - **A message bus** — append-log channel topics with durable retention, offsets,
>   acks, and replay. Delivery is the product; everything else is built on it.
> - **Hub-mediated** — a strict star: spokes never talk peer-to-peer.
> - **With terminal endpoints** — sessions are real PTYs: peers can stream output,
>   inject keystrokes, exec, and doorbell-wake them, not just exchange text.
> - **A coordination substrate** — presence, claim/lease work-stealing, DM threads,
>   and push-wake exist so N agents can divide and hand off work reliably.

Non-goals are guarded by *tripwires* (does the forbidden thing fail to build?).
Positive claims are guarded by *provers* (does the promised thing actually work?).
Only the first half had ever been audited.

> **The question:** for each thing the charter says TermLink *is*, does something
> affirmatively prove it — and is that proof ever run?

---

## Method

1. Decompose each load-bearing noun into individually falsifiable capability claims.
   "Sessions are real PTYs" is not one claim; it is four, and the charter lists them.
2. For each claim, find the artifact that would prove it, and read what that artifact
   actually exercises — not what its name or header says it covers.
3. For each prover found, grep the whole repo for anything that *invokes* it. A prover
   nobody runs is the T-2683 class, transplanted to the runtime tier.
4. Actively try to falsify the most suspicious claim rather than only confirm the
   comfortable ones — and record the result either way.

Step 4 matters for calibration. A review that only ever finds problems is not
measuring; it is performing. One suspicion in this pass was investigated and
**dissolved**, and that is reported with the same prominence as the findings.

---

## Finding F1 — the founding noun makes four claims and proves one

The charter's terminal-endpoints noun asserts peers can:

| # | Claim | Prover | Status |
|---|---|---|---|
| 1 | **stream output** | — | **none** |
| 2 | **inject keystrokes** | — | **none** |
| 3 | **exec** | `session-selftest.sh` EXEC + EXEC_EXITCODE | ✅ proven, and canary-wired (T-2557) |
| 4 | **doorbell-wake** | `comms-selftest.sh` (partial) | prover exists but is never run — see F2 |

`scripts/session-selftest.sh` is the affirmative prover for the founding verb —
"TermLink began as a cross-terminal session-control tool". Grepping it for the verbs
it exercises returns exactly one:

```
$ grep -oE "termlink [a-z-]+" scripts/session-selftest.sh | sort -u
termlink exec
```

Zero `inject`. Zero `output`.

### Why "inject has unit tests" is not an answer

`crates/termlink-session/src/handler.rs` does test inject — four tests, and their
names give the game away:

```
command_inject_resolves_keys_no_pty
command_inject_multi_entry_resolves_separately
command_inject_custom_delay
command_inject_unknown_key
```

`no_pty`. They prove *key-name resolution* — that `"Enter"` maps to the right byte
sequence — with no PTY attached. They cannot prove that a keystroke reaches a live
terminal and takes effect, which is the thing the charter claims. The remaining
`inject` callers in the tree are `demo-pushwaker*.sh`, `bench-pushwake-latency.sh`
and `wake-confirm.sh` — demos and benchmarks, not provers, and not wired to anything.

### The gap was known and written down

The most striking part is that `session-selftest.sh`'s own header states the problem
and then does not solve it:

> The gap was acknowledged in-code: `agent-conversation-selftest.sh` says "What it
> does NOT validate: PTY inject", and comms-selftest only checks the `pty_session`
> presence FLAG, never that a command actually injects and runs.

T-2485 correctly diagnosed that nothing proved PTY inject — and then built a prover
that uses `exec`. `exec` and `inject` are different primitives: `exec` runs a command
and captures its output; `inject` writes bytes into the PTY as if typed. Proving the
former says nothing about the latter. The acknowledgement was carried forward as
prose instead of being closed as a stage.

---

## Finding F2 — half the charter-verb provers are never executed

TermLink has four affirmative provers, one per charter verb:

| Verb | Prover | Executed by |
|---|---|---|
| 1 · discover | `comms-selftest.sh` | **nothing** |
| 2 · exchange durable messages | `comms-selftest.sh` | **nothing** |
| 3 · claim work | `substrate-smoke.sh` | **nothing** |
| 4 · control terminal sessions | `session-selftest.sh` | T-2557 canary ✅ |

Repo-wide grep for anything that *invokes* the first two returns only documentation,
handovers, task files, their own source, and — tellingly — **comments**.
`stuck-claims-canary.crontab` and `check-stuck-claims-freshness.sh` both mention
`substrate-smoke.sh`, but only in a header comment praising it as "the richest
affirmative prover"; the canary itself runs `claims-summary`.

This is T-2683's finding transplanted one tier up. That review established that the
*source-level* guard layer was executed by nothing and fixed it with
`run-guard-layer.sh`. It explicitly scoped out the runtime tier because those checks
need a live hub. The provers live in that excluded tier — and half of them turn out
to be in exactly the same state.

### Why this pass does not simply wire them to cron

`comms-selftest` needs a live **peer** to prove a round-trip; `substrate-smoke` needs
a reachable **hub**. Neither is guaranteed on an arbitrary host, so a naive cron
wiring would fire on *absence* rather than *breakage* — precisely the false-positive
class T-2557 avoided by splitting exit 2 (tooling) from exit 1 (finding), and
precisely the class T-2685 found had been destroyed anyway by `2>&1`. Wiring them
properly means giving each a "prerequisites absent → exit 2" contract first. That is
real work with a real design question in it, so it is **filed, not rushed**.

---

## Not a finding — "append-log" is true, and honestly documented

The most suspicious-looking claim in the charter is `append-log`, because
`channel edit` and `channel redact` both exist. An append-log with an edit verb
invites the obvious objection. It survives scrutiny:

- `cmd_channel_redact` posts a **new** envelope with `msg_type=redaction`, an empty
  payload, and `metadata.redacts=<offset>`.
- `cmd_channel_edit` posts a **new** envelope with `metadata.replaces=<offset>`. Its
  doc comment: *"Append-only: hub keeps the original; reader-side decides whether to
  render collapsed view. Old peers see two records."*
- The CLI names the verb **Retract**, not delete.
- The MCP descriptions say so unprompted: *"Append-only — the original envelope stays
  in the topic; reader-side aggregators decide whether to filter or render
  struck-through."*
- `channel redactions` returns *"a preview of the redacted payload"* — the surface
  goes out of its way to tell you the content is still there.

So the noun is accurate and the surface does not oversell it. Recorded with the same
prominence as the findings, because a review series that only ever produces problems
has stopped measuring and started performing. This is also a genuine contrast with
T-2690, where a README claim (`background` "Daemonizes with `setsid`" on macOS) was
found inaccurate and corrected — the difference between the two cases is evidence the
method discriminates rather than just accuses.

---

## Gap register

| # | Gap | Severity | Authority | Disposition |
|---|---|---|---|---|
| **G1** | `inject` — a charter-claimed PTY capability — has no end-to-end prover; unit tests are explicitly `no_pty` | **high** | agent | **BUILD** |
| **G2** | `output` streaming — likewise claimed, likewise unproven | medium | agent | **BUILD** |
| **G3** | `comms-selftest.sh` and `substrate-smoke.sh` are executed by nothing | medium | agent | **FILE** — needs a prerequisites-absent contract first |
| **G4** | Nothing keeps the noun↔prover matrix honest as claims change | medium | agent | **FILE** — sibling of T-2678's G4 (non-goal matrix), same open design question |

---

## Recommendation

**GO on G1 and G2.** Both are closable here and — unlike T-2692's macOS work, which
had to ship non-blocking because the platform could not be executed — both are
**verifiable on this host**: `tmux` is present and `session-selftest.sh --json`
currently returns `proven:true`. A prover stage that cannot be run is worth little;
these can be run, and will be, before the task closes.

**FILE G3 and G4.** G3 needs each prover to gain a "prerequisites absent → exit 2"
contract before any cron wiring, or it becomes a canary that fires on a quiet host.
G4 is the same open question T-2678 raised for the non-goal matrix and left for the
human; raising it a second time from the opposite direction strengthens the case but
does not change whose call it is.

---

## Assumptions registered

- **A1** — A tmux-backed scratch session can prove `inject` end-to-end by injecting a
  shell command plus Enter and then observing its effect. *Confidence: high*, and
  verified before close rather than assumed.
- **A2** — Proving `inject` via its observable effect (a sentinel appearing) is
  stronger than asserting the RPC returned ok. *Confidence: high.* An `ok` response
  proves the bytes were accepted, not that they reached a PTY — which is exactly the
  distinction the existing `no_pty` unit tests already fail to cross.
- **A3** — Adding stages does not slow the T-2557 canary meaningfully. *Confidence:
  medium.* The new stages reuse the session the prover already spawns; no extra spawn.

---

## Finding F3 — `inject` reported success for a complete no-op (found by building the prover)

This finding did not exist when the review began. It surfaced the moment an INJECT
stage was actually written — which is the strongest possible argument for F1.

Spiking the stage against a session with no PTY produced:

```
$ termlink inject <session> "echo INJECT-SPIKE-OK" --enter --json
{"bytes_injected":18,"ok":true,"target":"<session>"}
```

Nothing reached a terminal. `termlink status` reports `pty: null` for that session,
and its sibling `termlink output` refuses the same session **correctly**:

```
{"error":"JSON-RPC error -32007: No PTY session — output capture not available.
          Use `register --shell` for PTY-backed sessions.","ok":false}
```

That asymmetry is the proof it is a defect rather than a design choice: of the two PTY
verbs, one guards the precondition and the other does not.

### The lower layer was honest; the CLI threw the truth away

`handle_command_inject` (termlink-session `handler.rs`) distinguishes the two cases
explicitly — `status:"injected"` when a PTY took the bytes, and `status:"resolved"`
plus `note: "No PTY session. Use \`register --shell\` for PTY-backed injection."` when
there is none. Both are RPC successes. `cmd_inject` read only `result["bytes_len"]`
and discarded `status` and `note` entirely. The human path was worse than the JSON
one: it printed the word **"Injected"** for an operation that injected nothing.

### The fix already existed on the sibling surface

T-2580 made MCP's `termlink_inject` status-aware, extracted `mcp_inject_outcome` as a
pure helper, and pinned it with a test whose comment states the rule outright:

> a session with no PTY returns `status:"resolved"` … as an RPC *success* — but that
> is a FAILURE of the caller's intent (the command never reached a terminal), so it
> MUST NOT read "Injected successfully" (Reliability: no silent success on a no-op).

That reasoning was written down, tested, and never migrated to the CLI. This is the
"hardened in one place, siblings not migrated" class — the same shape as T-2666/T-2667
(silent exits) and T-2687 (topics inventory) — and it recurs because nothing compares
the two surfaces' *semantics*, only their JSON shapes, and the parity harness covers
24 of 68+ pairs (T-2689).

---

## Outcome

| Gap | Task | Result |
|---|---|---|
| G1 + G2 | **T-2695** | shipped — `session-selftest.sh` gains PTY_SPAWN, OUTPUT and INJECT stages. INJECT is proven **by effect**: the injected text embeds shell quoting the shell strips (`echo INJECT-PROVEN'-'<nonce>` typed, `INJECT-PROVEN-<nonce>` printed), so matching the unquoted string can only match the *interpreted* result — not the terminal's echo of the keystrokes. OUTPUT runs first so a broken observation channel is never misdiagnosed as broken inject. |
| F3 | **T-2697** | shipped — `inject_status_is_injected` pure helper + 3 tests (injected → true, resolved → false, missing/unknown/non-string → fail closed); refusal is loud in text *and* JSON and exits non-zero. |
| G3 | **T-2696** | filed at `horizon: next` — needs a prerequisites-absent (exit 2) contract before any cron wiring. |
| G4 | — | filed with T-2678's sibling; the noun↔prover matrix question stays with the human. |

**Verified on this host, not asserted:**

```
session-selftest --json → proven:true
  stages: spawn PASS · exec PASS · exec_exitcode PASS
          pty_spawn PASS · output PASS · inject PASS · cleanup done
check-session-control-freshness → healthy   (T-2557 canary still green)
guard layer → 23/23 clean
```

**Load-bearing, proven by sabotage:** replacing the injected sentinel with a
non-matching string yields `{"proven":false,"broken_stage":"INJECT","inject":"FAIL",
"output":"PASS"}` and exit 1 — the failure is isolated to inject, and OUTPUT still
passes, which is exactly the diagnostic precision the stage ordering was designed for.
Restoring returns it to green.

### A correction made mid-build

One acceptance criterion was written wrong and is struck through in the task rather
than quietly edited: *"both stages reuse the session the prover already spawns — no
extra spawn"*. The spike disproved it. STAGE 1 spawns `-- sleep <ttl>`, which has
`pty: null`; `output` refuses it with `-32007` and `inject` cannot reach a terminal
through it. Reuse was structurally impossible, so the PTY stages spawn their own
`--shell` session and the sleep-backed one is left untouched — which also means the
pre-existing stages, and the T-2557 canary that runs them daily, carry zero regression
risk from this change.

### Why this pass adds no new guard

Four of the five reviews so far ended by shipping a new check. This one ships none.
The finding was that a *claim* had no proof, so the fix is proof — extending the
existing prover rather than adding a sixth static check to a layer that already has
23 members. The one thing that would have been a new mechanism (wiring the two dark
provers to cron, G3) was **filed rather than rushed**, because doing it without a
prerequisites-absent contract would produce a canary that fires on a quiet host —
manufacturing exactly the false-positive class T-2557 designed around and T-2685 found
had been destroyed anyway.

---

## Dialogue Log

### 2026-08-14 — framing

**Human mandate (verbatim, 5th issuance):** *"please ultra critically review
termlink's purpose and goals and identify gaps or needed adjustement, incept these and
build these and test these, drive to comopletion"*

**How the axis was chosen.** With all four Directives now covered, the remaining
unexamined surface was the charter's own text: T-2678 audited the non-goals half and
nothing had audited the positive half. Decomposing "sessions are real PTYs" into the
four capabilities the charter itself lists — rather than treating it as one claim —
was what turned a vague noun into a testable matrix, and the matrix came back 1/4.

**On calibration.** This pass deliberately went hunting for a *false* charter claim
(`append-log`, given `edit`/`redact` exist) and found the claim sound and the
documentation candid. That result is reported at equal length to the findings. Four
consecutive reviews returning only problems would be evidence about the reviewer, not
the product; one returning a clean verdict on the thing it most suspected is evidence
the method discriminates.
