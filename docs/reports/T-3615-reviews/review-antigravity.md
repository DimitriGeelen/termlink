<!-- T-3615 external review, seat antigravity, source /tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/result.md, exit 0 -->
I will read `docs/reports/T-3615-fleet-telemetry-design.md` and the cited files in the repository to conduct the external review.
I will also read `runme.sh` (actions 1b and 23), `crates/termlink-session/src/audit_append.rs`, and `crates/termlink-session/src/discovery.rs` to check rotation, runtime directory resolution, and deployment details.
I will now read the remainder of `runme.sh` (lines 1601 to 2195) to inspect Action 23.
I will also check `.context/cron/telemetry-rollup.crontab`, `crates/termlink-cli/src/main.rs`, and `crates/termlink-mcp/src/server.rs` to verify how `invocation_audit::record` is called and how the local cron is configured.
I have completed my review of `docs/reports/T-3615-fleet-telemetry-design.md` and all cited source files (`crates/termlink-session/src/invocation_audit.rs`, `crates/termlink-session/src/audit_append.rs`, `crates/termlink-session/src/discovery.rs`, `crates/termlink-cli/src/main.rs`, `crates/termlink-mcp/src/server.rs`, `scripts/telemetry-rollup.sh`, `scripts/telemetry-fleet-view.sh`, `tests/telemetry-rollup-fixtures.sh`, and `runme.sh`). Note that the prompt specifically instructs: "Read-only: do not edit, build, run, post or contact any host other than GitHub... Print it as your final output; if you can write files, also write it to .context/working/t3615/review-<your vendor>.md and nothing else." Because the prompt specifies read-only and we are in `/plan` mode (which forbids modifying project files before approval), I am printing the complete review directly as my output below.

---

# External Review: TermLink Fleet Telemetry Design (T-3615)
**Reviewer:** Google Antigravity (Independent External Reviewer)  
**Target Commit:** `8e52cb0cf`

---

## 1. Objectives & Coverage of Non-TermLink Agents

**Does the design reach its objectives (§3)? No—coverage of non-TermLink agents has four structural blind spots.**

1. **Remote hosts deployed via Action 23 collect zero worker outcomes:** In [`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L2006-L2023), Action 23 installs `telemetry-rollup.sh` into `REMOTE_ROLLUP_ROOT="/var/lib/termlink-telemetry"` and runs `cd /var/lib/termlink-telemetry && bash scripts/telemetry-rollup.sh`. Inside [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L78-L144), `PROJECT_ROOT` resolves to `/var/lib/termlink-telemetry`. Consequently, `DISPATCH_DIRS` defaults to `/var/lib/termlink-telemetry/.context/dispatch:/tmp/tl-dispatch`, and `VERDICTS`/`APPLIED` resolve under `/var/lib/termlink-telemetry/.context/reviews/`. Because `/var/lib/termlink-telemetry` is an empty staging directory (created at [`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L2059)), every remote host (`.122`, `.121`, `.132`) misses all project `.context/dispatch` and `.context/reviews` directories unless Slice 4 item 6 explicitly configures multi-project discovery paths on remote hosts.
2. **Interactive and ad-hoc non-Anthropic seats that never call TermLink are completely invisible:** Section 3 Objective 1 and Section 4.4 claim the census (§5 item 7) and dispatch outcome records cover agents that never call TermLink. However, §5 item 7 builds the census *"from attributed rows, registered sessions and presence."* An interactive or CLI-launched Codex, OpenCode, Antigravity, or local Ollama session that neither goes through `fw termlink dispatch` nor registers a TermLink session nor invokes `termlink` leaves zero trace in any of those three sources.
3. **Ephemeral `/tmp/tl-dispatch` cleanup before the 02:30 UTC cron:** Dispatched workers running in `/tmp/tl-dispatch/<name>/` ([`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L141)) that do not call TermLink rely entirely on their `/tmp/tl-dispatch/<name>/` folder surviving until the next day's 02:30 UTC rollup. Any reboot, `systemd-tmpfiles --clean`, or dispatcher worktree pruning before 02:30 UTC permanently erases their run.
4. **Aggregate strips all dimensions except `class` from `usage`:** Objective 2 (§3.2) states every record says `class`, `harness`, `provider`, `model`, `worker_kind`, `worker`, `task`, `project`, `host`, and Objective 3 (§3.3) requires answering keep/cut/build and MCP vs. CLI across harnesses/providers. While [`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L328-L343) writes those fields to the local JSONL, [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L330-L336) aggregates usage strictly by `cls -> f"{surf}:{name}"`, discarding `harness`, `model`, `worker_kind`, and `project` (even asserting in [`tests/telemetry-rollup-fixtures.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/tests/telemetry-rollup-fixtures.sh#L156) that `harness` does not leave the host). As a result, fleet reports cannot tell whether `interactive` or `dispatched` CLI/MCP calls came from Claude Code, OpenCode, Codex, or Antigravity.

---

## 2. Attribution Model & Misclassifications

In [`crates/termlink-session/src/invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L275-L301), `attribution_with` evaluates a 6-step cascade. It will misclassify the following cases:

1. **Non-Claude agents executing tool calls are misclassified as `background`:**
   - Look at `agent_marked` ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L278)):
     ```rust
     let agent_marked = get("AI_AGENT").is_some() || get("CLAUDE_CODE_ENTRYPOINT").is_some();
     ```
   - When an interactive or standalone non-Claude harness (Codex, OpenCode, Antigravity, Aider) runs a shell tool subprocess (`termlink ...`), its subprocess `stdin` is a pipe (`stdin_tty == false`), and neither `AI_AGENT` nor `CLAUDE_CODE_ENTRYPOINT` is set (as noted in §4.5, *"other harnesses often set nothing"*).
   - If `TERMLINK_WORKER_CLASS` and `FW_DISPATCHED_WORKER` are unset, execution falls through to line 298:
     ```rust
     _ if !agent_marked && (systemd_service || !stdin_tty) => "background",
     ```
   - **Result:** Every `termlink` CLI call made by an interactive or non-`fw`-dispatched Codex/OpenCode/Antigravity agent is silently misclassified as `background` (grouped with cron canaries and watchers!).
2. **`termlink-mcp` server processes spawned by non-Claude harnesses are misclassified as `background`:**
   - An MCP server communicates over stdio pipes (`stdin_tty == false`). In [`crates/termlink-mcp/src/server.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-mcp/src/server.rs#L44-L47), `record(SURFACE_MCP, ...)` runs inside the MCP server process and caches `ATTRIBUTION` via `OnceLock` on first call ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L346-L354)). If the client harness does not set `AI_AGENT` or `CLAUDE_CODE_ENTRYPOINT` when spawning `termlink mcp serve`, all MCP tool calls from that harness are recorded as `class: "background"`.
3. **Interactive user shells inside systemd user scopes or piped commands:**
   - On systemd desktops/SSH sessions where `INVOCATION_ID` is inherited from a user service/scope without `FW_CLAUDE_FW_SUPERVISED`, a human running `termlink` at a terminal without `AI_AGENT` hits `!agent_marked && systemd_service` ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L298)) and is classified as `background` instead of `unknown` or `interactive`. Even without `INVOCATION_ID`, a human running `echo ... | termlink ...` or running `termlink` inside a script/subshell with redirected stdin has `!stdin_tty` and becomes `background`.
4. **Session-daemon (`SURFACE_SESSION_RPC`) attributes the *daemon*, not the *caller*:**
   - `ATTRIBUTION` is a process-wide `OnceLock<String>` ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L346-L354)). When the session daemon records `kv.*` or `session.*` calls ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L152-L160)), it stamps the *session daemon's* startup environment onto every RPC, regardless of which agent actually sent the RPC over the socket.
5. **Parent environment inheritance:**
   - If an `interactive` session or `standing` worker spawns a background daemon (`nohup ... &`) without explicitly overriding `TERMLINK_WORKER_CLASS=background`, the child inherits `AI_AGENT` and `CLAUDE_CODE_ENTRYPOINT=cli`, so all of its polling is misclassified as `interactive` or `standing`.

**How would you know?**  
From the fleet view, **you wouldn't**—because `telemetry-rollup.sh` strips `harness`, `worker`, and `project` from `signals.usage`. You could only detect this by inspecting raw host `invocation-audit.jsonl` rows or noticing suspiciously high non-polling verbs (`channel post`, `remote exec`, or `surface: "mcp"`) under `class: "background"`.

---

## 3. Worker-Outcome Measures

**Are they the right ones to judge "which worker is fit for which job"? No—several critical dimensions are missing or confounded:**

1. **`task_type` is dropped from the `dispatch` outcome key:**
   - In [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L369-L376), `cls` collapses `task_type` into binary `"reviewer"` (`task_type == "review"`) vs `"dispatched"` (everything else), and keys outcomes by `f"{cls}|{kind}|{model}"`. Build, debug, refactor, triage, and docs dispatches are all merged into `"dispatched"`. You cannot judge *"which worker is fit for which job"* when the job type (`task_type`) is erased from `worker_outcomes.dispatch`! (Meanwhile `close_ledger` at line 397 keys by `model|task_type` but omits `kind`/harness.)
2. **Defaulting missing `worker_kind.txt` to `"claude"` corrupts baselines:**
   - At [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L367):
     ```python
     kind = read_text(os.path.join(w, "worker_kind.txt")) or "claude"
     ```
   - Any dispatch directory where `worker_kind.txt` was not written (older dispatches, failed setup, alternative launchers) is falsely attributed to `"claude"`, skewing Claude's failure/run rates. It should default to `"unknown"`.
3. **Verdict join fails across days and across non-scanned directories:**
   - In [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L428-L444), `verdicts.jsonl` is filtered to rows whose timestamp falls in `[start, end)`. It joins `dispatch_id` against `dmeta` (populated from `DISPATCH_DIRS` at line 370). If the dispatch directory was in `/tmp/tl-dispatch` and already cleaned up, its verdict falls back to `("reviewer", "unknown", "unknown")`. Conversely, `applied.jsonl` is read in full (lines 417-423), so a verdict recorded on day $D$ and applied on day $D+1$ shows as `"pending"` on day $D$'s rollup (at 02:30 UTC) and is never retroactively updated unless day $D$ is re-rolled up.
4. **Standing worker `NRestarts` is a lifetime cumulative counter, not a daily delta:**
   - At [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L152) and [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L125), `systemctl show "$u" -p NRestarts` returns total restarts since the systemd unit was last reset, not restarts on `--day`. Every daily report repeats the cumulative number, making a historical crash look like ongoing daily instability.
5. **Missing fitness metrics:**
   - **Duration / Latency:** `meta.json` and `finished_at` exist, yet wall-clock run duration (`finished_at - started_at`) is never computed. A worker that takes 45 minutes is treated identically to one that takes 90 seconds.
   - **Cost / Token usage:** Zero tracking of input/output/cached tokens or dollar cost per run.
   - **Downstream rework / revert rate:** For `build` dispatches, `exit_code == 0` and non-empty `result.md` only prove the process exited cleanly and wrote text—not that tests passed, commits were kept, or the task wasn't reopened.
   - **Environment/Sandbox failure vs. Model failure:** As §2.7 notes, `opencode` returned an empty result because *the sandbox refused the checkout*, and the local model *could not run*. Counting infrastructure/sandbox misconfigurations as worker outcomes confounds harness bugs with model capability.

---

## 4. Privacy, Trust & Credentials

1. **Severe Credential Widening (Shared Hub Secret):**
   - Discovery 9 (§2.9) admits that remote fleet hosts (`.122`, `.121`, `.132`) hold `.107`'s full hub secret in their profile files. While T-3615 did not create those files, relying on a shared master HMAC secret means any compromise on `.121`, `.122`, or `.132` grants full administrative/RPC access to `.107`, not just append rights to `telemetry:daily`.
2. **Host Report Forgery & Overwrite Vulnerability:**
   - `telemetry-rollup.sh` sets `--host` from `$TELEMETRY_HOST` or `hostname` ([`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L59-L76)) and posts an unsigned JSON payload to `telemetry:daily` ([`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L491-L492)).
   - In [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L81-L84):
     ```python
     key = (body.get("host"), body.get("day"))
     if key not in reports or body.get("generated_at", "") >= reports[key].get("generated_at", ""):
         reports[key] = body
     ```
   - Because the hub does not bind `body["host"]` to an authenticated per-host identity key, **any caller on any host** with channel-post access to `.107` can post a report with `"host": "ring20-management"` and `"generated_at": "9999-12-31T23:59:59Z"`, permanently overwriting and poisoning another host's telemetry for any day.
3. **Topic Retention Eviction Attack ("Losing Another Host's Report"):**
   - Although `telemetry:daily` is created with `--retention days:400` ([`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L487)), any authenticated client can flood `telemetry:daily` with >5,000 messages. Because [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L44) reads with `--cursor 0 --limit 5000`, once the topic exceeds 5,000 envelopes, newer reports are silently truncated from the view!
4. **Local Multi-User Symlink / Sink Squatting in `/tmp`:**
   - In [`crates/termlink-session/src/discovery.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/discovery.rs#L25) and [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L136), fallback directories `/tmp/termlink-$UID` and `/var/tmp/termlink-$UID` have predictable names in world-writable sticky directories. And in [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L44), stderr is redirected to a hardcoded `/tmp/.telemetry-view.err`, which is vulnerable to symlink/FIFO attacks on multi-user systems (`testdev` exists on `.107`, per [`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L1478)).
5. **Privacy Leak in Proposed Slice 4 Census:**
   - While slices 1–3 strip `project` and `worker` from the posted aggregate, Slice 4 item 7 proposes sending a daily agent census containing `project` (derived from `PWD` basename in [`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L305-L313)) and distinct session/agent names. If an agent runs in a sensitive directory outside a project root, `basename(PWD)` leaks that directory name to the central hub.

---

## 5. Data Quality: Windows, Rotation, Polling, Observer Effect & 2-Week Horizon

1. **Polling floods the 32 MiB size cap, destroying real usage data before the daily rollup:**
   - `DEFAULT_MAX_BYTES` is 32 MiB with only a single `.1` backup (`~64 MiB` max total history) ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L143), [`audit_append.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/audit_append.rs#L13-L34)).
   - Polling verbs (`channel unread`, `pty output`, `ping`, etc.) are still written to `invocation-audit.jsonl` on every CLI/MCP invocation and only separated *at rollup time* ([`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L299-L336)).
   - With ~869k CLI calls/day (§2.2) at ~130 bytes/line (~113 MiB/day), the 64 MiB buffer (`invocation-audit.jsonl` + `.1`) holds **less than 14 hours of data**! By the time the cron job runs at 02:30 UTC ([`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L2022)), morning invocations from the previous UTC day have already been rotated into oblivion. Furthermore, `--catch-up 7` will produce severely undercounted partial reports for any missed day (`catch_up: true`), silently violating the module's core invariant ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L28-L30): *"The dangerous failure direction is therefore UNDERCOUNT: a tool that is used but reads as zero gets deleted"*).
2. **TOCTOU Rotation Race in `append_line_capped`:**
   - In [`crates/termlink-session/src/audit_append.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/audit_append.rs#L27-L35), `WRITE_LOCK` is a process-local `Mutex` ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L45-L48)). When two short-lived `termlink` CLI processes concurrently see `meta.len() >= max_bytes`, Process A renames `path -> path.1` and opens a new `path` and writes 1 line; Process B immediately renames that 1-line `path -> path.1`, **wiping out the entire 32 MiB `.1` backup!**
3. **Catch-up `POSTED_STATE` Bug with Offline Queue:**
   - [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L491-L496) appends `"$HOST $DAY_OUT" >> "$POSTED_STATE"` as soon as `termlink channel post` exits 0. As line 7 notes, `channel post` queues locally when `.107` is unreachable. If that queued message later expires or is dead-lettered, `POSTED_STATE` still claims the day was posted, so `--catch-up` will never re-send it.
4. **"A channel cannot report its own failures":**
   - `telemetry-fleet-view.sh` only flags `MISSING` for hosts it *knows* to expect (`hosts = sorted(set(expect) | {h for (h, d) in reports if d in window})` at [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L86)). Without a mandatory `--expect` list wired into an out-of-band alert, a host that never posts in the 7-day window disappears from `hosts` altogether!
5. **Is two weeks enough for the Q6 keep/cut/build analysis?**
   - **For cutting noisy polling or confirming high-frequency core verbs:** Yes.
   - **For deleting specialized CLI/MCP tools or judging worker fitness by job type:** **No.** Rare administrative/recovery tools (e.g., incident recovery, monthly/sprint review tools, specific arc tools) may legitimately have zero calls in a 14-day window. And for worker fitness stratified by `(kind, model, task_type)`, 14 days yields tiny sample sizes ($N < 5$ per cell) heavily skewed by whichever arc happened to be active during those two weeks.

---

## 6. Top 5 Changes Before Slice 4 is Built (Ranked by Impact)

1. **[Drop / Change] Stop writing high-frequency polling verbs to `invocation-audit.jsonl` (or split them to a separate counter/ring) and fix the multi-process rotation race.**
   - *Why:* Writing ~227k+ polling rows/day into a 32 MiB file rotates away the primary audit log in <14 hours and triggers the cross-process `rename` race in [`audit_append.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/audit_append.rs#L27-L34) that can wipe `.1`. Either filter or sample `POLLING` verbs in `invocation_audit::record`, or rotate daily by date (`invocation-audit-YYYY-MM-DD.jsonl`) instead of a lockless cross-process size rename.
2. **[Change] Fix the `background` heuristic in `invocation_audit.rs` so non-Claude agents and MCP servers are not swallowed as `background`.**
   - *Why:* Rule 5 ([`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L298)) marks any caller with `!agent_marked && !stdin_tty` as `background`. Since every agent's Bash tool and every MCP server has `!stdin_tty`, any harness that doesn't set `AI_AGENT` or `CLAUDE_CODE_ENTRYPOINT` (Codex, OpenCode, Antigravity, local models) gets misclassified as `background`. Restrict automatic `background` classification to known polling verbs or explicit `TERMLINK_WORKER_CLASS=background` markers, leaving unrecognized non-TTY callers as `unknown` (and check harness-specific env vars like `CODEX_*`, `OPENCODE_*`, `ANTIGRAVITY_*` or parent process tree).
3. **[Change] Fix remote host paths in Action 23 (`runme.sh`) and include `task_type`, `duration_s`, and `harness` in `telemetry-rollup.sh` aggregates.**
   - *Why:* Action 23 runs the rollup inside `/var/lib/termlink-telemetry` ([`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L2022)), where `.context/dispatch` and `.context/reviews` do not exist. Also, `telemetry-rollup.sh` must key `worker_outcomes.dispatch` by `class|task_type|kind|model`, include duration percentiles, default missing `worker_kind.txt` to `"unknown"` (not `"claude"`, line 367), compute daily deltas for `NRestarts`, and include `harness`/`model` in `signals.usage`.
4. **[Add] Sign telemetry rollup envelopes with per-host identity keys and paginate `telemetry-fleet-view.sh`.**
   - *Why:* Replace shared `.107` hub secrets on remote nodes with scoped/signed per-host identities so one host cannot spoof another's `"host"` field ([`telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L81-L84)), and paginate `channel subscribe` past `--limit 5000` (line 44).
5. **[Add] Instrument the census via process/harness discovery rather than TermLink-only state.**
   - *Why:* A census built only from *"attributed rows, registered sessions and presence"* (§5 item 7) will still miss non-TermLink agents (Objective 1). Scan `/proc` (or systemd cgroups / harness lockfiles) periodically or via dispatcher hooks to record non-TermLink agent processes.

---

## 7. Confidence & Verification Limits

| Question | Confidence | Basis & What Could Not Be Verified |
| :--- | :--- | :--- |
| **1. Objectives & Coverage** | **High** | Verified directly against [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L78-L144) and [`runme.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/runme.sh#L2006-L2023). Could not verify the external AEF repository's `fw termlink dispatch` implementation (lives outside this repo). |
| **2. Attribution Model** | **High** | Verified line-by-line in [`crates/termlink-session/src/invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L275-L354) and [`crates/termlink-mcp/src/server.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-mcp/src/server.rs#L39-L50). Could not inspect live environment variables on `.121`, `.122`, `.132`, or `node1`. |
| **3. Worker Outcomes** | **High** | Verified in [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L345-L465) and [`tests/telemetry-rollup-fixtures.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/tests/telemetry-rollup-fixtures.sh#L66-L95). Could not inspect live `.context/dispatch/*/meta.json` schemas produced by AEF across all projects. |
| **4. Privacy, Trust & Credentials** | **High** | Verified in [`scripts/telemetry-rollup.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-rollup.sh#L487-L498) and [`scripts/telemetry-fleet-view.sh`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/scripts/telemetry-fleet-view.sh#L44-L85). Could not inspect `/root/.termlink/hubs.toml` on remote fleet hosts (read-only local checkout constraint). |
| **5. Data Quality & Windows** | **High** | Verified rotation logic in [`crates/termlink-session/src/audit_append.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/audit_append.rs#L24-L38) and cap in [`invocation_audit.rs`](file:///tmp/tl-dispatch/t3615-rev-antigravity-dd4901bf5808/tree/crates/termlink-session/src/invocation_audit.rs#L143). Live daily byte volume was estimated from the 869k call count reported in §2.2. |
| **6. Top 5 Recommendations** | **High** | Grounded directly in the defects verified in sections 1–5. |
