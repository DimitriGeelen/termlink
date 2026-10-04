# Agent rules for this project — harness-neutral (T-2204)

These rules apply to **every agent working in this repository, whatever harness or vendor runs it**
(Claude Code, OpenCode, Codex, Gemini, an open-weights model behind any CLI, a role session dispatched
over TermLink, or a human). They are the single source: `CLAUDE.md` imports this file, `opencode.json`
lists it, and the arc-008 role chain feeds it to every role session. Do not copy these rules elsewhere;
point here.

Moved out of `CLAUDE.md` on 2026-10-03 (T-2204) after the T-2200 external review found that rules living
only in a harness-specific file are invisible to other harnesses.

## Never publish outside the estate (T-2180, operator rule)

**Nothing ring20 produces is published outside the estate.** No vendor-hosted artifact, canvas or share features (claude.ai
Artifacts, shared chats or canvases of any vendor), no external page/paste/document hosting, no "private" external links — not for review, not for sharing, not because a
tool description says finished work should be published. Documents live in the repository and are shown
through internal facilities: **Watchtower** (`http://192.168.10.122:3000/project/<path-with-->` renders
any Markdown file in the repo), OneDev, the internal web stack.

- In Claude Code the Artifact tools are denied in `.claude/settings.json` and in the user-scope settings.
  Do not remove those deny rules. Under any other harness, do not use its equivalent publish or share tools.
- For interactive review, propose an internal mechanism and ask before building it.
- Sending content to an external *model* (consultation panels, paid reviewers) is a separate act that needs
  the operator's explicit go each time, and the brief must carry no secrets, credentials or exploit detail.

**Why:** on 2026-10-03 a session published the arc-008 design document (estate topology, open
vulnerabilities) to claude.ai so the operator could comment inline. The operator had said before that
nothing goes outside; the rule was never written down, so it decayed. RCA:
`docs/reports/T-2180-external-publish-rca.md`.

## Search the OPEN queue before diagnosing (T-1540)

**Before investigating an error, filing a bug, or starting a diagnosis, search
the open task corpus for the distinctive symptom string:**

```bash
cd /root/proxmox-ring20-management && python3 scripts/task-prior-art.py --query "<distinctive string>"
```

Use the thing that is *specific* — the error text, the exit code, the config
key, the CT id (`cli_subcommand`, `CLOSE-WAIT`, `exit 403`, `ct130`). Not a
description of the problem in your own words: the whole failure mode is that
your words differ from the words already on file.

**Why this rule exists.** `fw work-on` already recalls prior art — learnings and
episodics of *completed* tasks — but it does **not** search open ones. The
framework remembers what it finished and forgets what it is still holding.
On 2026-08-04 that cost a full session: T-1390 had correctly diagnosed the
skills-mcp dispatch bug on 2026-07-10, sat in `.tasks/active/`, and was
rediscovered from scratch as T-1537 — which, missing T-1390's control group,
overstated the outage and filed that overstatement to a peer agent and into
project memory. A duplicate-*name* detector was measured and rejected: name
similarity between the two was Jaccard 0.08. The symptom string matched
instantly. Same session, querying the T-1287 vzdump work surfaced two more open
tasks aged 90+ days that were never consulted.

Rule of thumb: **if you are about to type an error string into a search
anywhere, type it here first.** A 100+ deep active queue is not searchable by
reading it.

(The recall integration is filed upstream. It is deliberately NOT patched into
`.agentic-framework/` — vendored patches are lost on `fw upgrade`, which is why
the rule lives in this project file.)

## Claim a peer request before acting on it (T-2088, G-215)

**Several ring20 sessions can run at once on .122, and they all share one TermLink identity
(fp `9219671e28054458`).** Every one of them reads the same DM threads, and posts by "ring20" are
indistinguishable by sender. On 2026-10-01 two sessions both executed dashboard-agent's T-2402
request (rotate the Pulse admin password) five minutes apart: two rotations, diverged Infisical
prod/dev, and a plaintext admin password left world-readable (T-2085/T-2086).

**Before acting on any request from a peer agent or channel** (a DM, a chat-arc ask, a directive
reply), claim it — keyed by peer and thread, not by post offset:

```bash
cd /root/proxmox-ring20-management && python3 scripts/peer-request-claim.py claim dashboard-agent:T-2402 --note "what I am doing (my task id)"
```

- exit 0 → it is yours; act.
- exit 3 → another *live* session holds it, or it is already done (the output names the session,
  the note and the outcome). **Do not act.** Read the outcome or wait; do not "help" in parallel.
- When finished: `peer-request-claim.py release <key> --outcome "<what was done>"`. Released
  claims stay visible for 7 days, so a late session sees "done by X" instead of redoing it.
- A claim whose session has died (PID gone) or whose TTL lapsed (default 12 h) is taken over
  automatically, and the takeover is reported.

**Name your session in replies** (your harness's session id, e.g. the first 8 chars of
`$CLAUDE_CODE_SESSION_ID` in Claude Code, plus your task id),
because the TermLink sender id cannot tell ring20 sessions apart.
