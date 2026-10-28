# T-3615 — Fleet telemetry: every agent, every harness, every provider

Status: design direction for external review (2026-10-28). Slices 1-3 built and pushed (c5ce50887, 913a6f5f8,
be5024645); slice 4 proposed below. Owner: operator (decisions), agent (build).

## 1. Statement

TermLink is the coordination substrate for one operator's fleet of AI agents (T-3612 round 1). To decide what to
keep, cut or build (value review T-3609, purpose review T-3612 Q6), and to judge which workers are fit for which
jobs, the operator needs evidence of what every agent actually does and how well every kind of worker performs:
one-shot `claude -p` workers, standing long-lived session workers, orchestration workers fired from a main agent,
interactive operator sessions, and non-Anthropic seats (OpenAI Codex, Z.ai GLM via opencode, Google Antigravity,
local models), on every host. Operator, 2026-10-28: "we don't have any data … we want to get data from all the agents
we can find in the fleet", "all workers!", "all the type of harnesses, all the type of providers".

## 2. Discoveries (measured 2026-10-27/28)

1. **Usage is recorded, per host, for both surfaces.** The termlink binary appends one row per CLI command and per MCP
   tool call to `<runtime_dir>/invocation-audit.jsonl` (`crates/termlink-session/src/invocation_audit.rs`; CLI
   `crates/termlink-cli/src/main.rs`, MCP `crates/termlink-mcp/src/server.rs`). MCP logging was proven live: one
   `termlink_version` call appeared at once.
2. **Zero MCP tool calls** over the logged window on .107 against 869k CLI calls, while sessions started 364 MCP
   servers. Possible readings: agents prefer the CLI, or cannot find MCP tools (Claude Code defers MCP tools: only
   names are shown until an agent searches for one). Not yet separated.
3. **About one day of history per host.** The sink rotates by size; polling verbs flood it (`channel unread` ~227k/day).
4. **Rows said nothing about the caller** (time, surface, name only), so a main session, a dispatched worker and a
   reviewer seat were indistinguishable.
5. **A stray, unread sink.** Dispatched workers do not carry `TERMLINK_RUNTIME_DIR`, so their rows landed in
   `/tmp/termlink-0` or `$TMPDIR/termlink-0` (26k rows, 12k on one day), outside every reader.
6. **The daily fleet rollup (T-3466) carried reliability signals only** (queue flaps, reconnects, breaker trips, dead
   letters, outages), no usage and no worker outcomes; only .107 reported. Its "missed" days: 10-24 the host was down
   (no catch-up existed); 10-27 a DST shift, not a miss.
7. **Worker quality is visible only per dispatch, never in aggregate.** `.context/dispatch/<name>/` and
   `/tmp/tl-dispatch/<name>/` hold meta, exit code, timeout and result; `dispatch-close-states.jsonl` (2,416 rows),
   `verdicts.jsonl` and `applied.jsonl` hold closes and reviewer verdicts. Example from 2026-10-27: of five purpose
   reviewers, opencode returned an empty result once (sandbox refused the checkout) and the local model could not run.
8. **After attribution went live (binary 0.12.1248):** of 33,829 attributed rows, 88% were `unknown`: background
   callers (cron canaries, sidecar watchers, wakers) with no agent marker. Hence the `background` class (slice 3).
9. **Fleet hosts already hold a credential for .107** (.122 `workstation-107`, .121 `framework-agent`, .132
   `designer107` profiles), the same route their sidecar mail uses, so posting reports needs no new secret. Side
   finding: those secret files may hold .107's full hub secret, which the project's own rule forbids on remote nodes
   (to be filed separately).

## 3. Objectives

1. **Coverage:** every agent that runs on any host of the fleet is counted, whatever its class, harness, provider
   or model, including agents that never call TermLink (census, objective 4).
2. **Attribution without content:** every record says who (class, harness, provider, model, worker kind, worker,
   task, project, host), never what was said.
3. **Usage that answers keep/cut/build:** per command and tool, per class and surface, with polling separated from
   intent, over windows long enough to judge (weeks, not one day).
4. **An agent census:** per host per day, which agents ran, of which class, harness, provider and model, one-shot or
   long-lived, first and last seen, restarts.
5. **Worker outcomes per kind and model:** runs, exit codes, timeouts, empty results, verdicts later applied or
   overturned on hand-check, standing-worker uptime and restarts, so "which workers are inferior for which job" is
   answered from data.
6. **Fleet completeness is visible:** a host that stops reporting is a named signal, and a late report is flagged.
7. **Nothing raw leaves a host;** only daily aggregates travel, through TermLink itself, never silently lost.

## 4. Considerations

1. **Privacy and scope.** Aggregates only; never message content, prompts, file paths of private work beyond the
   project name. Raw rows stay on the host and rotate.
2. **A channel cannot report its own failures.** Telemetry posted over TermLink cannot prove TermLink delivered it;
   a host that goes silent is the signal (fleet view MISSING), and an out-of-band check is still absent.
3. **Observer effect.** Our own canaries and watchers generate most rows; they must be classed `background` so they
   do not masquerade as use.
4. **Not every agent calls TermLink.** A Codex review that only reads files leaves no usage row; the census and the
   dispatch outcome records must cover it.
5. **Harness and provider identity is uneven.** Claude Code sets `AI_AGENT`; other harnesses often set nothing, and
   the model is rarely in the environment. The dispatcher knows the worker kind; it should pass kind, provider and
   model to the worker (an AEF change, since `fw termlink dispatch` is AEF's).
6. **Cost.** One env read per process, cached; one aggregate post per host per day (~7 KB measured).
7. **Credentials.** Reuse each host's existing profile for .107; no new secret. Do not post reports as sidecar mail
   (mail is surfaced to agents and would wake them daily); post to the `telemetry:daily` topic.
8. **Retention windows.** Raw rows ~1 day (size rotation); aggregates on `telemetry:daily` (bounded topic) are the
   history; long-term trends belong in a file or store outside the hub (T-3304 IW-1).
9. **Old binaries** keep writing unattributed rows until rolled out; the rollup reports them as `unattributed`.

## 5. Design direction

**Built (slices 1-3):**
1. Caller attribution on every record: `class` (interactive, dispatched, standing, reviewer, background, unknown),
   `harness`, `model`, `worker_kind`, `worker`, `task`, `project`; env only, computed once per process.
2. One sink per host: the hub's runtime dir wins over the stray `/tmp` fallback.
3. Daily rollup adds a `usage` aggregate (per class x surface x name; polling apart; stray sinks read once) and a
   `worker_outcomes` aggregate (dispatch runs, exit codes, timeouts, empty results, close states, verdicts
   applied/withdrawn, standing restarts or "not measured"); catch-up of missed days, flagged late.
4. `background` class via explicit markers in our crons and watchers plus a no-agent, no-tty heuristic.
5. runme action 23 installs the rollup on .122, .121, .132, verified on the host, posting to .107's
   `telemetry:daily` through each host's existing profile; fleet view shows per host and day.

**Proposed (slice 4):**
6. The rollup scans every project's dispatch folders on its host (not only TermLink's), so AEF, ring20, 1409 and the
   dashboard's workers are counted.
7. A daily agent census per host: distinct agents/sessions with class, harness, provider, model, project, first and
   last seen, one-shot vs long-lived; from attributed rows, registered sessions and presence.
8. `provider` derived from worker kind (claude→Anthropic, codex→OpenAI, opencode→Z.ai, antigravity→Google,
   local→ollama) when no explicit value exists; ask AEF to export kind, provider and model into every dispatched
   worker's environment.
9. The rollup on every host that runs TermLink, not only hubs (node1 via ring20).
10. After two weeks of fleet data: the Q6 analysis (keep/cut/build per command and tool; MCP vs CLI; worker fitness
   per job type), with decisions put to the operator.

## 6. Questions for reviewers

1. Does the design reach the objectives, especially coverage of agents that never call TermLink?
2. Is the attribution model sound (classes, markers, heuristic), and what will it misclassify?
3. Are the worker-outcome measures the right ones to judge worker fitness per job type? What is missing?
4. Privacy and trust: anything that leaks content or widens credentials more than it should?
5. What would you change, drop or add before slice 4 is built?
