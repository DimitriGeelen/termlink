# Profile: the TermLink project (010-termlink)

Estate specifics a role needs in this project. Given to a role together with its card and the AEF adapter.
Same headings as ring20's profile (`../upstream-ring20/profile-ring20.md`), from which this is adapted (T-3339).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | First profile, adopting ring20's role chain documentation layer (operator ruling C) | T-3339 |

## P1 Rules

P1.1 The project's agent rules: the project-specific sections of `CLAUDE.md` above `## Core Principle`
(everything below that line is framework-managed). There is no harness-neutral `policy/agent-rules.md` yet;
until there is, a non-Claude harness gets those sections as its rules file.
P1.2 Operator standing rules that bind every role's output and every message to the operator:
P1.2.a Number everything with unique hierarchical labels (1, 1a, 1ab), continued across the whole message, never
plain bullets (T-3329).
P1.2.b One operator decision at a time, with a recommendation, scored against both value-driver files
(`/decision-brief`); wait for "next" before the following decision.
P1.2.c Read operator words back before acting; they are often voice transcriptions.
P1.2.d Every command the operator must run goes into `/opt/termlink/runme.sh`, named by its full path; the
agent reads `.context/working/runme-logs/latest.log` itself.
P1.2.e Nothing is called working without a live test with two real agents and a negative control.
P1.2.f Never push to GitHub (OneDev `origin` only); never use a git worktree.
P1.3 The operator is the only human approver. Operator-facing text is plain English, numbered, with choices as
lettered lists A-D.

## P2 Where outputs are reviewed

P2.1 Step outputs are top-level files `docs/design/<design>-NN-<role>.md`. This project's Watchtower
(`.context/working/watchtower.url`, today `http://192.168.10.107:3003`) runs vendored AEF 1.6.29 and has **no
inline design-review page** (`/design/<doc>` returns 404). Until the re-vendor, the operator reviews the file in
the repository and answers in the session; the agent records the answer in the step task.
P2.2 The arc-011 design: the headline requirements
`docs/design/interactive-agent-communication-requirements.md`, the full design
`docs/design/interactive-agent-communication.md`, the traceability map
`docs/design/interactive-agent-communication-traceability.md`, the external reviews
`docs/reports/T-3335-design-review/` and `docs/reports/T-3335-routing-consult/`.

## P3 The estate

P3.1 Agents: this project's sessions (010-termlink; several may run at once), the framework agent
(999-Agentic-Engineering-Framework, AEF), 055-agentic-fleet-cockpit, 832-Workflow-designer, ring20-manager
(proxmox-ring20-management, hub .122), ring20-dashboard (hub .121), Penelope. Peers are reached by their project
inbox on the hub (`inbox:<hub-id>/<project>`); a peer that cannot read this repository gets documents posted as
messages (832, project boundary).
P3.2 Protected systems: the TermLink hubs of the fleet (.107, .121, .122, .141) and their secrets and TLS pins;
the OneDev repository and its GitHub mirror and release pipeline; the cron canaries in `/etc/cron.d`.
P3.3 Known incidents and weaknesses to use as evidence: the 2026-10-03 receive-side incident (mail stored and
receipted, never surfaced: arc-011); G-058 (silent mirror failure), G-060 (per-hub topics, the origin of
"hubs never talk"), G-063 (write-only sinks), G-069 (shipped but not live), T-2396 (input typed into a busy
prompt lost); 055's E1-E7 and M1-M6 in its review reports. Gaps live in `.context/project/concerns.yaml`.

## P4 Tools in this estate

P4.1 Code is Rust (crates/) plus bash and Python 3 scripts. Tests: Rust `cargo test --workspace`; hermetic
fixture suites `tests/*fixtures*.sh` and `tests/test-*.py` (PASS/FAIL lines, exit code); the guard layer
`scripts/run-guard-layer.sh`.
P4.2 Browser evidence: Playwright (MCP in Claude Code), screenshots read in every visual mode affected.
P4.3 Orchestration: TermLink (sessions, channels, claims, `fw termlink dispatch` for separate sessions). Role
sessions get messages and artifacts only, never keystroke injection into other sessions.
P4.4 External review: `codex exec -s read-only` (OpenAI) and `opencode run -m zai-coding-plan/glm-5.3` (Z.ai)
with the brief inlined and the questions after the context, stdin closed (`< /dev/null`); answers stored
unedited with a comparison. Local models (ollama qwen3, gemma4) proved too weak for design review (T-3335).
P4.5 Not available until the re-vendor: `fw reviewer judge` and `fw reviewer verdict apply` (the rung-5 panel),
and therefore ring20's driver `role-chain.py`. Step order and approval are kept by the orchestrator by hand.
