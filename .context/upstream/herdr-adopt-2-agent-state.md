# Herdr adoption #2 — agent-state detection: is a protocol/heuristic HYBRID worth having? (2026-08-15)

**Recommendation: DO NOT RECOMMEND** a heuristic fallback wired into
`agent-presence` / `find-idle` / `/peers`. A narrow, *separately-named,
non-authoritative* observational verb is defensible but nobody has asked for it
(§7).

**Tooling note:** Bash, Grep, Read and WebFetch were all available for this
investigation. Unlike `herdr-internal-tmux-surface.md`, the absences below are
grep-established across the stated paths, not bounded to files-I-happened-to-open.
Every absence claim names the command that produced it.

**Licence:** everything below is **category (a) — idea only**. No herdr code is
proposed for copying. The manifest *format* is quoted as evidence, not adopted.
Any future work that vendors `src/detect/manifest.rs` or the `.toml` manifests
becomes **category (b)** and needs attribution + NOTICE handling (Apache-2.0 at
v0.8.0).

---

## 1. How herdr's manifest format actually works

Read at **pinned tag `v0.8.0`** — `raw.githubusercontent.com/herdrdev/herdr/v0.8.0/...`.
The docs page `herdr.dev/docs/agents/` does **not** publish the schema (fetched;
it gives location + prose only). The schema is only in source.

**Files.** 20 bundled manifests at `src/detect/manifests/*.toml`, compiled in via
`include_str!` (`manifest.rs:239`). A remote/website copy at
`website/agent-detection/` with an `index.toml` (`schema_version = 1`, `[[agents]]`
`id`/`path`). Local override at
`~/.config/herdr/agent-detection/<agent>.toml` (`manifest.rs:1097-1103`).

**Manifest header** (`AgentManifest`, `manifest.rs:139-150`, `deny_unknown_fields`):
`id`, `version`, `min_engine_version`, `updated_at`, `aliases[]`, `rules[]`.

**Rule** (`ManifestRule`, `manifest.rs:153-180`):

| Field | Meaning |
|---|---|
| `id` | rule name, echoed in `explain` output |
| `state` | `idle` \| `working` \| `blocked` \| `unknown` (`ManifestState`, :216-222) |
| `priority` | i32; highest matched rule wins (:443-446) |
| `region` | which slice of screen to match (default `whole_recent`, :235-237) |
| `visible_idle` / `visible_blocker` / `visible_working` | confidence flags for source arbitration |
| `skip_state_update` | rule matched but must NOT change state (transient UI) |
| `contains[]` | case-insensitive substring set |
| `regex[]` / `line_regex[]` | regex over whole region / per-line |
| `all[]` / `any[]` / `not[]` | recursively nested `ManifestGate`s (:184-198) |

**Regions** (`validate_region_name`, :1072-1095) — 12 named plus 3 parameterised:
`whole_recent`, `after_last_prompt_marker`, `before_current_prompt_marker`,
`whole_recent_without_current_prompt_marker`, `current_prompt_block_marker`,
`after_current_prompt_block_marker`, `prompt_box_body`, `above_prompt_box`,
`last_non_empty_above_prompt_box`, `after_last_horizontal_rule`, `osc_title`,
`osc_progress`, plus `bottom_lines(N)`, `bottom_non_empty_lines(N)`, `top_lines(N)`.

**Evaluation** (`evaluate_loaded_manifest`, :414-446): every rule is evaluated
against its region; among matches, the highest `priority` wins (ties keep the
first). Complexity is capped: 128 rules, gate depth 8, 512 gates, 1024 matchers,
512 chars per matcher (:264-269).

**Worked example** — `claude.toml` (`version = "2026.07.13.1"`):

```toml
[[rules]]
id = "bash_permission_prompt"
state = "blocked"
priority = 850
region = "whole_recent"
visible_blocker = true
contains = ["do you want to proceed?"]
any = [{ contains = ["bash command"] }, { contains = ["ctrl+e to explain"] }, ...]
all = [{ any = [{ line_regex = ['(?i)^\s*❯?\s*yes\b'] }, ...] }]
```

That is a literal string-match on Claude Code's permission-prompt chrome.
`osc_title_working` (priority 1100) matches a **braille spinner glyph range**
`^[\x{2800}-\x{28FF}]` in the terminal title.

**Two corrections to the brief's framing:**

1. **There is no `done` state.** `AgentState` is `Idle | Working | Blocked |
   Unknown` (`src/detect/mod.rs:11-20`). Nothing reports completion.
2. **The failure mode is worse than "new prompts misclassify".** It is
   structural and it fails *open*. `fallback_explain` (`manifest.rs:497-534`):

```rust
state: if known_agent {
    AgentState::Idle          // manifest.rs:529
} else {
    AgentState::Unknown
},
...
fallback_reason: known_agent.then(|| DEFAULT_KNOWN_AGENT_IDLE_FALLBACK...)
```

**When a recognised agent matches NO rule, the answer is `idle`.** A stale
manifest, a redesigned prompt, a resized pane that pushes chrome out of
`bottom_non_empty_lines(5)`, an agent mid-tool-call between spinner frames — all
report `idle`, which in dispatch terms means *"available, send it work."* The
lie is biased toward the most expensive direction.

**Also:** manifests are fetched remotely (`read_remote_manifest` :722,
`manifest_update.rs`, `remote_update_status`). State classification can change
from a network fetch without a local code change — a D4 consideration.

---

## 2. Our side — what TermLink actually has (verified)

**Presence carries no agent state at all.** The complete metadata vocabulary on
`agent-presence` is `{agent_id, role, capabilities, pty_session, cv_key}`
(`crates/termlink-bus/src/lib.rs:600-613`, `crates/termlink-session/src/fleet_presence.rs:101-218`,
producer `scripts/listener-heartbeat.sh:173-190`).

I checked for a state field rather than assuming its absence:

```
grep -rnE 'metadata[^)]*\b(get|insert)\("(state|status|busy|activity|blocked|working|idle)"' crates/ scripts/
→ 0 matches
```

Zero here is real and I verified it by enumerating what *is* consumed (the
positive list above), not just by the negative grep.

**Our "idle" is a different quantity from herdr's.** `find_idle_agents`
(`lib.rs:558-655`) computes:

> `LIVE(agent-presence)` minus `DISTINCT(active claimers)` — `lib.rs:643-647`

with LIVE = `cutoff_ms < ts ≤ future_cutoff_ms` (:631, symmetric skew bound per
T-2536) and `msg_type == "heartbeat"` (:597, T-2585). `/peers` uses the same
definition: `age <= 2*interval => LIVE` (`scripts/agent-listeners.sh:296`).

So **TermLink `idle` = "heartbeating AND holding no claim."** It is a *bookkeeping*
fact derived from two protocol records. **herdr `idle` = "the bottom of the screen
looked like a prompt, or nothing matched."** These are not two implementations of
one predicate. TermLink cannot distinguish working from blocked *at all* — that
axis does not exist in our model — and herdr cannot tell you whether an agent
holds a lease.

The raw material for a heuristic *is* present: `query.output` RPC over
`ScrollbackBuffer` (`commands/pty.rs:43,104,202`; `scrollback.rs:7`) already
returns screen text from an unrelated process. So the hybrid is *buildable*. It
should still not be built into these three surfaces.

---

## 3. The decisive objection: it breaks the liveness contract by construction

A non-opted-in agent emits no heartbeat. To make it visible to `/peers` or
`find-idle`, something must **synthesise an `agent-presence` heartbeat the agent
never sent**. Follow that through T-2387:

`check-waker-liveness-freshness.sh:121-146` counts `live_total` for every entry
with `status == "LIVE"`, and `live_armed` only for those carrying `pty_session`.
Class **(a) LIVE-but-unwakeable** fires on `LIVE ∧ ¬pty_session`.

A scraped agent has no waker and no `pty_session` — it never opted in; that is the
entire premise. So **every synthesised entry fires class (a), permanently, and no
operator action can clear it.** Its remediation ("relaunch through the T-2388
launcher") is exactly the opt-in the agent declined.

This repo just fixed this precise bug elsewhere. T-2709 narrowed the stuck-claims
canary off `expired_count > 0` because it was *"a monotonic latch… so it fired
daily on permanent debris — the precise mechanism by which a guard teaches its
operator to stop reading it"* (CLAUDE.md §Stuck-claims canary). A presence
fallback re-introduces that latch into the waker canary — the guard for the comms
rail. **Rejecting this is consistency with a decision already made, not caution.**

Secondary, still disqualifying:

- **False-idle feeds dispatch.** `find-idle` is the input to `/claim` →
  `/claim-transfer` (CLAUDE.md §orchestrator recipe). herdr's fail-open (§1)
  means the *common* failure reports "available". Work gets assigned to an agent
  blocked on a permission prompt, and the lease then expires into the stuck-claims
  canary — a second guard absorbing damage from the first. D2 (no silent failures)
  violated at the dispatch layer.
- **`/peers` LIVE would stop meaning one thing.** T-2585 (`lib.rs:589-596`)
  deliberately unified the liveness predicate across all three discovery surfaces
  because disagreement was a "trust-the-topic reliability gap". A second, weaker
  producer re-forks it.
- **Cost is weeks, not days.** The engine is ~1,500 LOC (`manifest.rs`) plus a VT
  grid for `prompt_box_body` / `after_last_horizontal_rule` region extraction,
  plus 20 manifests to track against upstream churn. Writing it ourselves is
  category (a) but expensive; vendoring theirs is category (b).

---

## 4. What herdr genuinely has that we do not

Stated plainly, because it is real: **herdr can tell you an agent is *blocked
waiting on a human*, and TermLink cannot** — not for opted-in agents either. Our
model has no blocked/working axis (§2). That is a genuine capability gap and the
honest reason the hybrid question is worth asking.

It is just not a *presence* gap, and answering it through the presence rail is
what makes the proposal unsafe.

---

## 5. Recommendation table

| What | Adopt? | Evidence | We already have? | Effort | Directive |
|---|---|---|---|---|---|
| Heuristic fallback into `agent-presence` / `find-idle` / `/peers` | **NO** | `manifest.rs:529` fail-open-to-idle | presence has no state field (`lib.rs:600-613`, grep §2) | weeks | fails D1, D2 |
| Nested `all`/`any`/`not` gate grammar as a design idea | **NO — nothing to apply it to** | `manifest.rs:184-198` | n/a | n/a | — |
| `skip_state_update` (matched-but-don't-transition) as a design idea | **worth remembering** | `manifest.rs:167`, `claude.toml` `transcript_viewer` | n/a | 0 (idea) | D2 |
| Separate, non-authoritative observational verb (§7) | **defer — unasked** | — | `query.output` exists (`pty.rs:43`) | ~1 wk | D3 |

## 6. The one idea worth keeping

`skip_state_update` (`manifest.rs:167`): a rule that matches, is reported in
`explain`, and explicitly **declines to change state**. herdr uses it for the
Claude transcript viewer and model picker — UI that proves nothing about whether
work is happening. It is the same instinct as our exit-2 tooling class
(T-2557: *"a check that never looked must never read as a clean bill"*). Worth
carrying forward if we ever build an inference surface. No code, no licence
exposure.

## 7. The only defensible version, and why I am not proposing it

If the blocked-detection gap in §4 is ever worth closing, the shape that does not
break anything is: a **separate verb** over sessions TermLink *already owns PTYs
for* — read `query.output`, classify, report under its own name with its own
confidence field. Never written to `agent-presence`. Never read by `find-idle`.
Never counted by any canary. Zero coupling to the liveness contract.

That is category (a), buildable, and roughly a week. **I am not recommending it**,
because it serves no requested need, and CLAUDE.md's own charter review (T-2468)
found TermLink *over-built in breadth* with T-2548 still open on ~28 off-charter
tools. Adding an inference surface nobody asked for, while a human decision to
*subtract* surface is pending, is the accretion the charter-drift canary (T-2483)
exists to catch. If it is ever wanted it should enter as an inception task with a
human go/no-go — not as a fallback smuggled into a rail three canaries depend on.

---

## 8. Not established

- I did not run herdr. All behaviour is read from source at `v0.8.0`; I did not
  empirically confirm the fail-open path fires in practice, only that the code
  path exists and is unconditional for a known agent with no matching rule.
- Detection accuracy rates: no measurement exists upstream and I made none. My
  objection does not depend on the rate — it depends on the *direction* of the
  error (§1) and on the canary latch (§3), both of which hold at any accuracy
  below 100%.
- I did not audit the remote-manifest fetch path (`manifest_update.rs`) for
  signature verification. Flagged in §1 as a D4 consideration, not costed.
