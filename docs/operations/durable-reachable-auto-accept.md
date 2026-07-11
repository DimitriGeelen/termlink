# Durable, launch-path-independent auto-accept for reachable agents (T-2401)

A **reachable** agent (one that opted into agent-presence via `/be-reachable` or
`tl-claude --reachable`) must be able to **post a reply hands-free** when a peer's
doorbell wakes it — otherwise it is discoverable + wakeable but **mute**, and the
comms loop dies after one hop. This doc records how to make that auto-accept
**durable across any relaunch**, which is the third leg of a reliable reachable
agent.

## The three legs of a reliable reachable agent

Deploying (or re-onboarding) a reachable agent means getting **all three** right.
Missing any one silently breaks agent-to-agent comms:

| Leg | What it guarantees | Where it lives | Durable across `claude --resume`? |
|---|---|---|---|
| **1. Binary** | The `termlink` on PATH speaks the current wire (identity resolver, push-waker, etc.) | `~/.cargo/bin`, `~/.local/bin`, `/usr/local/bin` — install to **all** PATH shadows | n/a (on-disk) |
| **2. Identity** | The session **signs** replies as its own per-agent key, not the shared host key | `.mcp.json` → `mcpServers.termlink.env.TERMLINK_AGENT_ID` (T-2399) | ✅ Claude Code injects it into the `mcp serve` it spawns |
| **3. Auto-accept** | A woken agent **posts** the reply with no permission prompt | `.claude/settings.local.json` → `permissions.allow` (**this doc**, T-2401) | ✅ honored from settings, no CLI flag |

Legs 2 and 3 are the two halves of the same problem — the split that produced the
original "comms silently stops" incidents. Leg 2 (T-2399) made **identity**
durable via `.mcp.json`. Leg 3 (T-2401) makes **auto-accept** durable the same
way: via project config Claude Code reads on every session start.

## Why not `--dangerously-skip-permissions` / `bypassPermissions`?

- The **CLI flag** (`tl-claude --reachable` injects it, T-2400) works but is
  **launch-path-dependent** — a plain `claude --resume` that omits it comes back
  up mute. That is exactly how agents regress.
- `permissions.defaultMode: "bypassPermissions"` in **project** settings is
  **ignored** by Claude Code (guarded since v2.1.142 so a repo cannot self-grant
  bypass). It only works in the **user** `~/.claude/settings.json`, which is not
  portable per-project. So it is not a durable per-agent answer.
- A **scoped `permissions.allow` list is honored from project settings with no
  CLI flag**, survives `claude --resume`, and is narrow (only the comms path —
  the agent is still gated on unrelated file/bash ops). This is the durable fix.

## The exact block

Merge these into each agent-project's `.claude/settings.local.json`
`permissions.allow` (idempotent — add only if absent):

```json
{
  "permissions": {
    "allow": [
      "Bash(termlink channel:*)",
      "Bash(termlink agent:*)",
      "mcp__termlink__termlink_channel_*",
      "mcp__termlink__termlink_agent_*"
    ]
  }
}
```

Why all four:

- Agents reply via the **MCP** `termlink_channel_*` tools (post / ack / subscribe
  / unread / list / reply / thread). The wildcard covers the whole receive→reply
  loop in one entry (Claude Code allows a tool-name glob after the literal
  `mcp__<server>__` prefix; the server segment must be glob-free).
- Some flows (`/check-arc respond`, `/reply`, `agent-respond.sh`) shell out via
  **Bash** to the `termlink channel` CLI — `Bash(termlink channel:*)` covers those.
- `termlink_agent_*` / `Bash(termlink agent:*)` cover `agent contact`,
  `find-idle`, and other agent verbs used by the handoff skills.

### The trap this closes

Before T-2401 the allow-lists were **incomplete and inconsistent** — e.g. several
agents had `channel_post` but not `channel_unread`/`channel_list`/`channel_ack`,
and one agent (sonnenstall) had **no channel tools at all**. They only worked
because they were launched with the blanket `--dangerously-skip-permissions`
flag. The moment such an agent is relaunched with a plain `claude --resume`,
`/check-arc`'s `channel_list`/`unread` calls prompt → the unattended agent stalls.
The wildcard entry makes the list complete-by-construction, so no individual verb
can be missing.

## Applying / re-applying

Idempotent merge across the .107 fleet projects (T-559-safe via `termlink run`,
backs up to `*.pre-t2401`): see the script in T-2401's Updates, or run the same
`json.load → ensure entries → json.dump` loop over each project's
`.claude/settings.local.json`.

Verify per project:

```bash
python3 -c "import json; a=json.load(open('.claude/settings.local.json'))['permissions']['allow']; \
print(all(e in a for e in ['Bash(termlink channel:*)','mcp__termlink__termlink_channel_*']))"
# -> True
```

## Onboarding checklist (new reachable agent)

1. **Binary** — install current `termlink` to every PATH shadow.
2. **Identity** — add `TERMLINK_AGENT_ID=<id>` to `.mcp.json`
   `mcpServers.termlink.env` (T-2399).
3. **Auto-accept** — merge the block above into `.claude/settings.local.json`
   (this doc).
4. Launch via `tl-claude start --reachable --agent-id <id>` (T-2400 also injects
   the CLI flag as a belt-and-suspenders for the current session).
5. Confirm: `/peers --all` shows the agent LIVE with its **own** fingerprint (not
   the shared host key) and `pty_session` set.

## Related

- T-2399 — MCP identity leak fix (leg 2, `.mcp.json` identity).
- T-2400 — `tl-claude --reachable` auto-accept (leg 3, launch-time form).
- T-2401 — this doc: durable settings-based auto-accept (leg 3, launch-independent).
- `docs/migrations/T-1700-per-agent-identity.md` — per-agent identity keys.
- CLAUDE.md § "Shared-host envelope identity" / project memory
  `reference_shared_host_identity`.
