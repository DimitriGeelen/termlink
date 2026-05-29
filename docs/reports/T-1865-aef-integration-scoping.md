# T-1865 — AEF integration scoping: doorbell+mail propagation vs explicit coordination

**Status:** spikes complete, recommendation written
**Filed:** 2026-05-29
**Question (verbatim from human):** "do we need to coordinate with framework agent
for integrating the doorbell+mail workflow into aef, or will this go automatically
when termlink is called?"

## Hypothesis under test

Two distinct layers, likely different answers:

| Layer | Pre-spike hypothesis | Outcome |
|---|---|---|
| Binary-level rail (`termlink` CLI, hubs, MCP server) | Auto-propagates via brew/GitHub releases | Confirmed (already known) |
| Operator toolkit (`scripts/*.sh`, `.claude/commands/*.md`, systemd presence templates) | Lives in `/opt/termlink`, does NOT auto-propagate to AEF consumers | **Confirmed by Spike 2** |
| AEF awareness (does upstream know we exist?) | Probably no integration; framework-agent coordination needed | **Confirmed by Spike 1** — pattern exists, content absent |
| Coordination channel (framework-agent reachable?) | Cohort DMs via d1993c2c shared identity, last activity T-1166-era | **Worse than feared (Spike 3)** — channel dormant, no framework-agent presence |

## Spike 1 — AEF upstream surface scan

Path correction: upstream lives at `/opt/999-Agentic-Engineering-Framework`,
not `/opt/999-AEF`. Memory updated.

`ls /opt/999-Agentic-Engineering-Framework/.claude/commands/`:

```
capture.md, deploy-check.md, explore.md, new-project.md, plan.md,
resume.md, review.md, rollback.md, start-work.md, write.md
```

10 framework-default skills. **Zero doorbell+mail skills.** No
`/be-reachable`, no `/peers`, no `/recent-chat`, no `/recent-dm`,
no `/agent-handoff`, no `/check-arc`, no `/broadcast-chat`, no
`/pulse`, no `/conversations`.

`ls /opt/999-Agentic-Engineering-Framework/scripts/`:

```
spikes
```

Only a `spikes/` subdirectory. No production scripts. **No
`agent-chat-arc-recent.sh`, `recent-dm.sh`, `agent-listeners-fleet.sh`,
`chat-arc-broadcast.sh`, `agent-conversation-list.sh`, or
`agent-send.sh` upstream.**

**Verdict:** AEF has the PATTERN for framework-shipped skills (the
`.claude/commands/` directory and the 10 existing skills prove it),
but does NOT contain any doorbell+mail toolkit. A2/A3/A4 all
confirmed.

## Spike 2 — vendored-AEF sync mechanism (`fw upgrade` semantics)

Read `/opt/termlink/.agentic-framework/bin/fw` line 254-264 — the
canonical `do_vendor` includes list:

```bash
local includes=(
    bin
    lib
    agents
    web
    docs
    .tasks/templates
    FRAMEWORK.md
    metrics.sh
)
```

`scripts/` is NOT in this list. `.claude/commands/` is NOT in this list.

Read `lib/upgrade.sh` line 172: `fw upgrade` does sync ONE specific
skill: `.claude/commands/resume.md`. Read lib/upgrade.sh line 1020-1051:
this is a special-cased copy from `lib/templates/`, separate from
do_vendor. The other 9 upstream skills (plan, explore, capture, etc.)
also live upstream but are NOT vendored down to consumer projects —
the framework keeps them as dev-time tools for working ON the framework
itself.

**Verdict:** Even if doorbell+mail skills were added to upstream
`.claude/commands/`, they would NOT reach consumer projects via
`fw upgrade` unless EITHER:

- (a) we extend `do_vendor`'s `includes` list to add `.claude/commands/`
  + `scripts/` (structural change to fw), OR
- (b) we add per-skill special-cased copy paths to `upgrade.sh` like
  the existing resume.md path (mechanical change), OR
- (c) consumers manually copy the toolkit (current default for the
  upstream's other 9 skills).

This bifurcates the answer to A2/A3: SCRIPTS are 100% consumer-local
today, SKILLS are 100% consumer-local today except for resume.md.

## Spike 3 — coordination channel check

Live readings on 2026-05-29:

- `bash scripts/agent-listeners-fleet.sh --include-offline --json | jq -r '.listeners[].agent_id' | sort -u`
  → returns ONE listener: `root-claude-dimitrimintdev` (me).
  No framework-agent / aef-agent / skills-manager-agent / cohort-agent
  presence visible on the fleet.

- `bash scripts/recent-dm.sh d1993c2c --since 168 --limit 5 --filter-msg-type chat`
  → 30 dm topics involving the shared-host fingerprint, but only ONE
  chat post in the last 7 days. That post (2026-05-26T07:07:31Z) is from
  sender d1993c2c (this host) referencing T-1166.

**Verdict:** Even if framework-agent IS the right counterparty (A5),
the doorbell+mail rail cannot deliver the coordination message because
framework-agent has no LIVE listener and the rail is essentially silent
from their side. A5 is the only assumption that remains unanswerable
from local-readable evidence — needs a direct upstream-side commit or
GitHub PR, not a DM.

## Synthesis

**Binary-level rail:** auto-propagates. Brew + GitHub releases ship
new termlink versions; consumer projects upgrade `termlink` itself
via standard package paths. When a project invokes `termlink agent
contact <name>` or `termlink channel post agent-chat-arc <text>`,
those primitives work today on any host that has the binary +
hubs.toml + secret deployed. **No AEF action needed for this layer.**

**Operator toolkit:** does NOT propagate. The 9 doorbell+mail slash
skills + the 7 supporting scripts + the systemd presence-emitter
template all live ONLY in `/opt/termlink`. They do NOT exist in
upstream AEF, and `fw upgrade` does NOT include `.claude/commands/`
or `scripts/` in its vendoring contract. **Every AEF consumer
project that wants this toolkit must obtain it some other way.**

**Coordination channel:** dormant. framework-agent has no live
heartbeat on the fleet, the DM channel is quiet (1 post / 7 days,
from this side). Even if they are the right counterparty for the
vendor decision, the rail cannot reach them — coordination has to
go via upstream commit (termlink_run can do this; memory has the
pattern) or GitHub.

**The full answer to the user's question:** it will **NOT** go
automatically when termlink is called. The binary works automatically,
yes — but the operator toolkit (the verbs operators actually use to
participate in doorbell+mail) is termlink-project-local and won't
reach AEF consumers without explicit coordination. AND the natural
coordination counterparty (framework-agent) is currently absent from
the rail, so the coordination cannot itself happen via doorbell+mail —
it needs upstream commit access.

## Recommendation

**GO** on a follow-up build (small, scoped) to bring the operator
toolkit into AEF. The simplest path is also the right one:

- **Phase 1 (one task, ~1 session):** Extend upstream AEF with a
  doorbell+mail skill bundle. Upstream gains
  `.claude/commands/{be-reachable,peers,recent-chat,recent-dm,broadcast-chat,pulse,conversations,check-arc,agent-handoff}.md`
  + `scripts/{agent-chat-arc-recent,recent-dm,agent-listeners,agent-listeners-fleet,chat-arc-broadcast,agent-conversation-list,agent-conversation-status,agent-send,listener-heartbeat}.sh`
  via direct upstream commit (termlink_run path, memory documented).
  These are FILE COPIES with no behavioral change.

- **Phase 2 (one task, ~1 session):** Extend `do_vendor` includes to
  add `.claude/commands/` + `scripts/` to the vendoring contract,
  so existing AEF consumers gain the toolkit on next `fw upgrade`.
  This is the structural framework change — it should be reviewed
  carefully because it changes upgrade semantics for every consumer
  project, not just doorbell+mail.

- **Phase 3 (optional, follow-on):** docs + per-host operator runbook
  for hubs.toml setup, secret deployment, systemd presence opt-in.

**No DEFER needed.** The coordination channel being dormant is not a
blocker — we can ship Phase 1 directly via upstream commit, then
notify framework-agent retroactively when they next come online (the
update will be visible to them as a normal upstream change).

**Concrete next tasks if approved (NOT created until GO decision):**

1. **T-1866** (build) — vendor doorbell+mail skill bundle + script
   bundle into upstream `/opt/999-Agentic-Engineering-Framework`.
2. **T-1867** (build) — extend `do_vendor` includes to add
   `.claude/commands/` + `scripts/` so `fw upgrade` propagates them to
   consumer projects. Requires careful review — affects every consumer.
3. **T-1868** (docs, optional) — operator runbook for hub deployment
   + presence opt-in + identity setup.

The pair T-1866 + T-1867 together resolves the user's question
structurally: after they ship, the doorbell+mail toolkit WILL go
automatically when consumers run `fw upgrade`.

## Evidence index

- Upstream skill bundle present: spike 1 ls output
- Upstream scripts absent: spike 1 ls output
- `do_vendor` includes list: `bin/fw:255-264` (Read)
- `fw upgrade` skill sync limited to resume.md: `lib/upgrade.sh:172`
  (line: "  - .claude/commands/resume.md"), corroborated by line
  1020-1051 (the only special-cased copy path for skills)
- Fleet listeners as of 2026-05-29: one agent_id only
- Cohort DM channel chat-msg volume: 1 post / 7 days
