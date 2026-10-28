# T-3615 — Synthesis of three external reviews of the fleet telemetry design

Inputs: the design `docs/reports/T-3615-fleet-telemetry-design.md` and three reviews in `docs/reports/T-3615-reviews/`:
**C** = `review-claude.md` (Anthropic), **X** = `review-codex.md` (OpenAI), **G** = `review-antigravity.md` (Google).
A fourth seat, opencode (Z.ai), failed in its sandbox and produced no review. The Codex dispatch ended with exit 1, but its
review is complete. Antigravity printed its review instead of writing a file. Every finding below was checked against the
code at HEAD on .107. "True" means the cited code does what the reviewer says.

## 1. Agreed findings (raised by two or three reviewers)

| # | Finding | By | Checked | Impact |
|---|---|---|---|---|
| A1 | The census (slice 4 item 7) is built from attributed rows, registered sessions and presence, so it cannot see an agent that never calls TermLink | C, X, G | True by the design's own text (§5 item 7) | Objectives 1 and 4 cannot be met as designed |
| A2 | An agent with no marker and stdin that is not a terminal is classed `background`. That catches non-Claude agents' shell tools, MCP servers started by non-Claude harnesses, and humans piping into termlink | C, X, G | True, `invocation_audit.rs:278,298` | The undercount the module itself calls dangerous (`:28-31`): real use is discounted as observer effect |
| A3 | The usage aggregate keeps only class, surface and name. Harness, model, kind, worker, task and project are dropped, and the join to dispatch meta promised at `invocation_audit.rs:110-112` does not exist | C, X, G | True, `telemetry-rollup.sh:330-334` | "Which harness or model uses what" cannot be answered once raw rows rotate |
| A4 | On remote hosts the rollup runs from `/var/lib/termlink-telemetry`, so the project's dispatch, verdict and applied files are never read | C, X, G | Partly true. `runme.sh:2007,2022` with `telemetry-rollup.sh:78,141-144`. `/tmp/tl-dispatch` is still read | Remote worker outcomes are partial and verdicts are always `absent` |
| A5 | Rotation cuts days short and nothing says so. A catch-up of older days posts near-zero usage | C, X, G | True. Only the live file and `.1` are read (`:309`), and `sources` says `present` even when 0 rows fall in the day (`:338-339`). On .107 today, `.1` spans 18.0 h and the live file 6.6 h: about 24.6 h in total | Short days read as low use, which is the wrong direction for "cut" decisions |
| A6 | Two processes that rotate at once can overwrite the `.1` backup | C, G | True. `audit_append.rs:27-33`, and the mutex is process-local (`invocation_audit.rs:45-47` says so) | About 32 MiB of rows lost silently |
| A7 | `session-rpc` rows carry the session daemon's identity, not the caller's | C, G | True. `handler.rs:165-169` records the call using the daemon's cached attribution (`invocation_audit.rs:346-353`) | `kv.*` and `session.*` use is credited to whoever registered the session |
| A8 | Attribution is frozen per process and inherited. An explicit class overrides everything, and the model falls back to `ANTHROPIC_MODEL` | C, X, G | True, `invocation_audit.rs:280,317,346` | A leaked `background`, or a stale task or model, is recorded as fact |
| A9 | `kind` defaults to `claude` when `worker_kind.txt` is missing | C, X, G | True, `telemetry-rollup.sh:367`. On .107 only 31 of 922 dispatch folders have the file | Non-Claude failures inflate Claude's numbers |
| A10 | Applied/withdrawn is frozen on the day the verdict is recorded and never revised | C, X, G | True. `:409-443`, and catch-up only fills days that are missing (`:111`) | Daily reports mostly show "pending" |
| A11 | "Empty result" is judged by file size alone | C, X | True, `:384` | A refusal message counts as output |
| A12 | Fitness measures are missing: duration, tokens/cost, rework or revert, and job type. Job type is collapsed to reviewer/dispatched | C, X, G | True, `:369`. `close_ledger` keeps `task_type` but drops kind (`:397`) | "Fit for which job" cannot be answered |
| A13 | Each report's host and timestamp are self-declared, and the latest `generated_at` wins, so one report can mask another host's | C, X, G | True, `telemetry-rollup.sh:65,76,492`, `telemetry-fleet-view.sh:81-84` | Data integrity, and the view can be poisoned by accident (a bad clock) as well as on purpose |
| A14 | The fleet view reads a single page (`--cursor 0 --limit 5000`) | C, X, G | True, `telemetry-fleet-view.sh:44` | Newer reports silently vanish once the topic grows past 5,000 |
| A15 | A host that never reported is invisible unless `--expect` is passed, and an empty fleet returns ok | C, X, G | True, `:86,134` | Objective 6 is unmet |
| A16 | Reusing a possibly full .107 hub secret on remote hosts widens credentials; posting needs a scoped credential | C, X, G | Not checkable here. The design says "may hold" (§2.9); G overstates it as "admits" | Compromising one host gives full access to .107, not only append rights |
| A17 | `NRestarts` is a lifetime counter, reported as if it were per day | X, G | True, `:152,455`. The view sums it (`view:125`) | An old crash looks like ongoing instability |
| A18 | Deriving `provider` from worker kind is a guess (opencode can front any provider) | C, X | Holds as reasoning against design §5 item 8 | Wrong provider attribution |
| A19 | Two weeks is not enough to cut rare tools or to judge worker fitness | C, X, G | Judgement. C counted 16 Codex and 10 opencode runs in total | A premature "cut" |
| A20 | Polling is decided by verb name, so deliberate `list` and `hub status` calls count as polling | C, X | True, `:299-301` | Intent is undercounted |
| A21 | `/tmp/tl-dispatch` is wiped on reboot, so a crashed host loses its failures first | C, G | True by path (`:141`) | Survivorship bias |
| A22 | The `PWD` basename fallback for `project` yields junk names and can leak private folder names, and slice 4 would export it | C, G | True, `invocation_audit.rs:302-313` | Privacy |

## 2. Single-reviewer findings

**Hold up when checked:**

1. C: the append makes two writes, the line and then `\n` (`audit_append.rs:36-37`), so concurrent writers can interleave. This contradicts the file's own atomicity claim at `:6-8`. The same writer serves the hub's `rpc_audit` (`:1-4`).
2. C: non-root callers cannot write to a root-owned hub directory, and the error is swallowed (`invocation_audit.rs:396,402`). **On .107, `/var/lib/termlink` is `drwx------ root`**, so every non-root caller's rows are lost. Remote hosts are unverified.
3. C: the rollup resolves the runtime directory differently from the binary (`telemetry-rollup.sh:120-123` vs `invocation_audit.rs:230-245`), and as root it reads only `/tmp/termlink-0` among the old fallback sinks (`:131-139`).
4. C: `sdk-*` entrypoints are classed `dispatched` (`invocation_audit.rs:297`), which also catches interactive SDK and IDE use.
5. C: model names are not normalised (`opus` 548, `claude-opus-5-5` 47). This data count was not re-run here.
6. X: the aggregate exports each sink's full file path (`telemetry-rollup.sh:337`), standing unit names (`:454`) and reconnect peer identifiers (`:241`). That violates design §4.1. Fixture U8 only checks that two sample values are absent (`tests/telemetry-rollup-fixtures.sh:156`); it does not enforce an allowlist.
7. X: dispatches with no finish timestamp are skipped (`:371-373`), so crashed or hung launches are never counted.
8. X: dispatch meta is keyed by directory name only (`:370,429`). Once slice 4 scans several projects, same-named folders will collide or be counted twice.
9. X: an agent-marked systemd one-shot is classed `standing` before its SDK entrypoint is considered (`:291` comes before `:297`).
10. X: the classes mix lifecycle (standing), launch mechanism (dispatched) and role (reviewer), so a standing reviewer cannot be expressed (`:248`).
11. G: the posted-days file is written as soon as `channel post` exits 0 (`:491-496`). The post can only be queued (header `:7-8`), so a day later dead-lettered is never re-sent by catch-up.
12. G: the view writes stderr to a fixed `/tmp/.telemetry-view.err` (`view:44`), a symlink target on a multi-user host. Low risk.

**Do not hold, or are overstated:**

13. G says the log holds "<14 h". That assumes 869k calls per day, but design §2.2 gives 869k over the logged window. Measured today it is about 24.6 h (A5). The direction of the finding is right; the size is wrong.
14. G says interactive shells in systemd scopes carry `INVOCATION_ID` and become `background`. systemd sets that variable for service units, not normally for login shells. The piped-stdin part does hold (A2).
15. G frames paging past 5,000 posts as an "eviction attack". The paging bug is real (A14); the attack needs a hostile poster inside a one-operator fleet.

## 3. Disagreements

16. **How much history the log holds:** G says under 14 h, C says 24-27 h. **C is right**: 24.6 h measured on .107 (`/var/lib/termlink/invocation-audit.jsonl{,.1}`).
17. **How to stop polling flooding the log:** G would stop logging polling verbs one by one, or rotate by date; C would count hourly on the host; X would keep incremental local summaries with coverage metadata. All three would work; this is operator question Q1.
18. **What to do with unmarked, non-terminal callers:** G would class them `unknown` and keep `background` for explicit markers only; C would detect each harness's markers and cross-check against a `ps` snapshot; X would record which rule fired and a confidence. See Q2.
19. **Provider:** C would drop the derivation; X would keep it, labelled as a hypothesis. See Q4.
20. **Credentials:** C proposes a post-only token scoped to the topic; G per-host signing keys; X host-scoped rights bound at ingestion. Envelopes already carry a `sender_id` (`crates/termlink-bus/src/envelope.rs`). Whether the hub binds it to the posting credential was not checked by any reviewer, nor here. See Q3.
21. **Window for keep/cut:** G says two weeks is enough for polling but not for rare tools; C asks for four weeks of zero use with complete data on every host; X asks for complete observation plus enough comparable tasks. See Q6.

## 4. Revised slice 4 (ranked by impact)

| # | Item | Size | Touches |
|---|---|---|---|
| 22 | **Fix data loss first (section 5 items 30-33) before any expansion.** Slice 4 only multiplies undercounted data | small-medium | TermLink |
| 23 | Make the census harness-side: scan processes for harness binaries and read the harnesses' own session logs, independent of TermLink. Until that exists, call it an "observed-agent census" (X) | needs design | TermLink |
| 24 | Launchers export class, kind, provider and resolved model ID into every worker's environment (design §4.5; C, G) | medium | AEF (upstream ask) + TermLink |
| 25 | A shared run record keyed by a stable run ID: project, job type, kind, resolved model, duration, tokens/cost, `fallback_used`, launch failure, unfinished. Deduplicate across projects before scanning more than one (replaces item 6 as written; A12, items 7-8) | needs design | AEF (meta schema) + TermLink |
| 26 | Score outcomes over a trailing window after a set maturation delay (applied verdicts, rework, reverts) instead of freezing daily snapshots (A10; C, X) | medium | TermLink |
| 27 | Trust before rollout: verify the remote credentials, use a scoped post credential, bind the host to the sender, use hub timestamps (A13, A16) | needs design | TermLink (hub authz) |
| 28 | An authoritative host inventory used as the default `--expect`, plus an out-of-band pull of each host's posted-days file (A15; C, X) | small | TermLink |
| 29 | Item 9 (every TermLink host) and item 10 (Q6 analysis) stay, gated on complete-data weeks. Item 8 (provider from kind) is dropped or marked "inferred" (Q4) | small | TermLink |

## 5. Fixes to the built slices 1-3 (ranked)

30. Write one line per syscall, and lock rotation across processes, or use per-day files (item 1, A6). Small; TermLink crate; also fixes `rpc_audit`.
31. Non-root rows are lost on .107 (item 2). Make the sink group-writable, or give each user its own sink that the rollup also reads. Medium; TermLink.
32. Coverage flags: per sink, compare the oldest kept row with the day's start and mark the day complete or partial. Report `present` only when rows exist, and post `usage: null` on a rotated catch-up (A5). Small.
33. Remote rollup paths: point dispatch, verdict and applied at the real project directories (A4). Match the binary's runtime-directory resolution (item 3). Small.
34. Privacy: replace sink paths with labels and drop or hash peer and unit names (item 6). Turn fixture U8 into an export allowlist. Small.
35. Keep harness, kind and model in the usage aggregate, bucketed, with "other" beyond an allowlist (A3). Small.
36. Outcomes: kind defaults to `unknown` (A9); add job type to the key (A12); count unfinished runs (item 7); report restarts as a daily delta (A17). Small.
37. Fleet view: page past 5,000 posts; an empty fleet is not ok; on a re-run, prefer the complete report over the latest one (A14, A15; X). Small.
38. Attribution: `session-rpc` takes the caller's identity (A7); the order of the `standing`/`sdk` rules (item 9). Medium (needs caller identity in the RPC).
39. Mark a day posted only once delivery is confirmed (item 11). Use `mktemp` for the view's stderr file (item 12). Trivial.

## 6. Questions for the operator (take one at a time)

40. **Q1 — Polling volume.** A: do not log polling verbs one by one; keep counters instead (G). B: count hourly on the host (C). C: keep local daily summaries with coverage metadata (X). D: only raise the cap.
41. **Q2 — Unmarked, non-terminal callers.** A: class them `unknown`, and keep `background` for explicit markers only (G; raises "unknown" from the post-slice-3 level). B: keep the heuristic, add harness-marker detection and a `ps` cross-check (C). C: record the rule that fired and a confidence with each row (X).
42. **Q3 — Credential before action 23 runs unattended.** A: a post-only token scoped to `telemetry:daily` (C). B: per-host signing keys (G). C: bind the host to the authenticated sender at the hub (X). Also: pause action 23 until the remote `hubs.toml` contents are verified? All three reviewers say verify before expanding.
43. **Q4 — Provider from worker kind.** A: drop it (C). B: keep it, marked "inferred" (X).
44. **Q5 — Census scope.** A: build a harness-side census by process scan and harness logs (C, G). B: ship the TermLink-only census, named "observed agents", and state the gap (X).
45. **Q6 — When Q6 keep/cut may run.** A: after two weeks, for polling and high-volume verbs only (G). B: after four weeks of complete data on every host (C). C: once each comparison has enough complete samples, whatever the time (X).
