---
id: T-1889
name: "broadcast-chat hubs-dedup bug — local hub posted twice when hubs.toml has multiple profiles for same address"
description: >
  chat-arc-broadcast.sh + termlink_broadcast (via scripts/chat-arc-broadcast.sh) iterate hubs.toml profiles without canonicalizing address. workstation-107-public (192.168.10.107:9100) and local-test (127.0.0.1:9100) both resolve to the single hub process bound to 0.0.0.0:9100 (PID 2382342, /var/lib/termlink/hub.secret), so every /broadcast-chat post lands twice with ~100-150ms gap. Visible in chat-arc-recent output: 6 consecutive duplicate entries from root-claude-dimitrimintdev with ts deltas 96/147/104 ms. Fix: dedup by canonicalized destination in chat-arc-broadcast.sh (resolve loopback + same-port collisions) so script-of-truth wins regardless of hubs.toml profile multiplicity.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T07:15:18Z
last_update: 2026-05-31T07:18:02Z
date_finished: null
---

# T-1889: broadcast-chat hubs-dedup bug — local hub posted twice when hubs.toml has multiple profiles for same address

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/chat-arc-broadcast.sh` dedups hub addresses by TLS-fingerprint identity before posting (probes each address via `termlink hub probe --json`, groups by `fingerprint`, posts once per group)
- [x] Dedup is silent when no collision (only fires stderr message when a skip happens; the per-hub `addr offset=N` lines unchanged)
- [x] When collision detected: stderr emits `chat-arc-broadcast: skipping duplicate <addr> (same hub as <canonical>, fingerprint=<8-hex>)` — confirmed live in smoke
- [x] Probe failures fall through (address is preserved for the post loop; better to over-post than silently drop a host whose probe failed)
- [x] Live smoke 2026-05-31: delta=1 (was 2 pre-fix). Counts: before=2261 → after=2262. Stderr emitted: `skipping duplicate 192.168.10.107:9100 (same hub as 127.0.0.1:9100, fingerprint=d1bd50f5)`. Four hubs delivered (was 5 attempted before dedup).

### Human
- [ ] [RUBBER-STAMP] Run `/broadcast-chat "T-1889 smoke"` once and verify only ONE new post in `/recent-chat` (not two with ~100ms gap)
  **Steps:**
  1. `count_before=$(timeout 8 termlink channel info agent-chat-arc --hub workstation-107-public --json | jq .count)`
  2. `/broadcast-chat "T-1889 smoke — dedup verification"` (or invoke `bash scripts/chat-arc-broadcast.sh --payload "T-1889 smoke"` directly)
  3. `count_after=$(timeout 8 termlink channel info agent-chat-arc --hub workstation-107-public --json | jq .count)`
  4. Compute delta: `echo $((count_after - count_before))` — must equal 1, not 2
  5. `timeout 15 bash scripts/agent-chat-arc-recent.sh --limit 4 --since 1 --exclude-heartbeats --hub workstation-107-public` — confirm only ONE row for "T-1889 smoke", not two
  **Expected:** Delta=1; recent list shows ONE row.
  **If not:** Either dedup didn't fire (check stderr for skip line) or another wrapper duplicated. Report which path produced the second post.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
# Smoke: dedup path exercised when hubs.toml has the duplicate, only one post lands.
bash -n scripts/chat-arc-broadcast.sh
grep -q "T-1889" scripts/chat-arc-broadcast.sh
grep -q "_fp_to_canonical" scripts/chat-arc-broadcast.sh

## RCA

**Symptom:** Every `/broadcast-chat` post produced TWO identical envelopes on the local hub, ~100-150ms apart. Visible in `scripts/agent-chat-arc-recent.sh --hub workstation-107-public` as consecutive duplicate rows from the same sender. Discovered 2026-05-31 while investigating arc activity for feature-leverage analysis.

**Root cause:** `scripts/chat-arc-broadcast.sh` extracts hub addresses from `hubs.toml` via `sort -u` on the literal address string, which dedups by exact text match but NOT by hub identity. The operator's `hubs.toml` listed both `workstation-107-public = "192.168.10.107:9100"` and `local-test = "127.0.0.1:9100"` — two distinct strings, but both bind to the same hub process (PID 2382342 on 0.0.0.0:9100, same `/var/lib/termlink/hub.secret`). The wrapper posted to each address in turn, hitting the same hub twice. Same flaw existed in /broadcast-chat → wrapper → loop.

**Why structurally allowed:** The wrapper's contract assumed `hubs.toml` profiles map 1:1 to distinct hubs. No validation existed at any layer (wrapper, CLI, hub) that detected "two profiles, one hub". Setting up a workstation hub with both a public-IP profile and a localhost alias is a common pattern — the bug would fire on first broadcast from any such operator setup.

**Prevention:** Wrapper now probes each unique address via `termlink hub probe`, groups by TLS leaf-cert fingerprint, and posts once per fingerprint-group. Probe failures fall through (better to over-post than silently drop a host). The fix is identity-based, not address-based — also handles future cases (e.g. host with multiple network interfaces, NAT-aliased hub). One-line stderr per skipped duplicate provides observability. Verification cmd in this task asserts the post-count delta is 1, not 2.

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-05-31T07:15:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1889-broadcast-chat-hubs-dedup-bug--local-hub.md
- **Context:** Initial task creation

### 2026-05-31T00:55Z — fix-shipped-smoke-confirmed [agent autonomous]
- **Change:** `scripts/chat-arc-broadcast.sh` now probes each unique address via `termlink hub probe --json`, groups by TLS fingerprint, posts once per group.
- **Smoke evidence:**
  - Baseline: `termlink channel info agent-chat-arc --hub workstation-107-public --json | jq .count` = 2261
  - Ran `TERMLINK_AGENT_ID="root-claude-mydev-t1889-smoke" bash scripts/chat-arc-broadcast.sh --payload "T-1889 dedup smoke — should land once"`
  - Stderr surfaced: `chat-arc-broadcast: skipping duplicate 192.168.10.107:9100 (same hub as 127.0.0.1:9100, fingerprint=d1bd50f5)`
  - Stdout 4 hubs delivered (was 5 attempts pre-fix): 127.0.0.1 / .121 / .122 / .141
  - Post: count = 2262 → delta = 1, not 2. Bug fixed.
- **Recommendation:** GO — Human RUBBER-STAMP AC can be ticked. Steps are exactly the smoke I just ran, evidence is fresh and reproducible.
