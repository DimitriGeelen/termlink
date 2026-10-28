<!-- T-3615 external review, seat claude, source .context/dispatch/t3615-rev-claude-b813f66dbd63/result.md, exit 0 -->
# T-3615 external review — Anthropic (Claude Opus 5.5)

I reviewed commit 8e52cb0cf read-only and wrote the review to `.context/working/t3615/review-anthropic.md`. I read the cited files and looked at local data on .107 (the audit log files and a read-only count of `/tmp/tl-dispatch/*/meta.json`). I built nothing, ran no fixtures and contacted no other host.

## 1. Objectives: partly met; agents that never call TermLink are not covered

- **Coverage and census (objectives 1 and 4) are not built, and the planned sources can't provide them.** Slice 4 builds the census from attributed rows, registered sessions and presence. All three exist only for agents that use TermLink.
  - A Codex or Antigravity seat that only reads files leaves no row.
  - Dispatch folders cover only agents started by `fw termlink dispatch`. A `codex` or `opencode` run by hand, or an IDE session, appears nowhere.
  - A real census needs a source on each host: a process scan for harness binaries, or the harnesses' own session logs.
- **Attribution (objective 2) is collected and then thrown away.** Every row carries `harness`, `model`, `worker_kind`, `worker`, `task` and `project` (`invocation_audit.rs:328-343`). The rollup counts only class and command name (`scripts/telemetry-rollup.sh:330-336`). The module doc promises a join to the dispatch's meta (`invocation_audit.rs:111-112`), but that join doesn't exist.
- **Completeness (objective 6):** the view lists only hosts that have reported at least once (`telemetry-fleet-view.sh:86`). A host that never reported is invisible unless `--expect` is passed.

## 2. Attribution: what it will misclassify

1. **Non-Anthropic agents land in `background`.** Rule 5 (`invocation_audit.rs:298`) means "no agent marker and stdin not a terminal". That is exactly the shell tool of Codex, opencode or Antigravity when fw didn't dispatch it. The design tells readers to discount `background` as observer effect (§4.3), so their real use gets counted as noise. The module doc names this undercount as the dangerous failure (`:28-31`).
2. **`session-rpc` rows get the session daemon's identity, not the caller's.** The daemon records the call itself (`handler.rs:165-169`) using its own cached environment (`invocation_audit.rs:349-353`). Every `kv.*` and `session.*` call is credited to whoever registered the session.
3. **The MCP server's attribution is frozen when it starts** (`invocation_audit.rs:346`), so a task change mid-session is never seen.
4. **`project` falls back to the basename of `PWD`** (`:305`). That gives junk projects such as `tmp`, and can expose private folder names.
5. **`sdk-*` entrypoints are counted as `dispatched`** (`:297`), which also catches interactive SDK and IDE use.
6. **An explicit `TERMLINK_WORKER_CLASS` overrides everything** (`:280`). If a crontab's `background` value leaks, any agent launched from that environment inherits it.

**How you would know:** you wouldn't, today. Add a daily cross-check against a `ps` snapshot of harness processes, and track and alert on the `background`/`unknown` share per host.

## 3. Worker-outcome measures

The current measures (`telemetry-rollup.sh:353-458`) show whether a worker ran, not whether its output was good.

**Missing:**
- duration;
- tokens and cost (available in `result.jsonl`);
- rework (a task dispatched again, or a commit reverted);
- `fallback_used`, which is in meta and ignored;
- finer task types. Of 887 metas, 801 are `build`, so "fit for which job" can't be answered.

**What will mislead:**
- **Model names aren't normalised.** I counted `opus` 548, `claude-opus-5-5` 47 and `sonnet` 253. An alias such as `opus` changes meaning when the model is upgraded.
- **`kind` defaults to `claude` when `worker_kind.txt` is missing** (`:367`). Only 30 of 889 dispatch folders have that file.
- **Applied/withdrawn is fixed on the day a verdict is recorded** (`:427-443`). Verdicts are applied later, so daily reports mostly show "pending" and never catch up.
- **"Empty result" is judged by file size** (`:384`), so a refusal message counts as a success.
- **Samples are tiny:** 16 Codex and 10 opencode runs in total.
- **Survivorship bias:** `/tmp/tl-dispatch` is wiped on reboot, so a crashed host loses its failures first.

## 4. Privacy, trust and credentials

- **Content leak** is low today, since only command names travel. Slice 4 adds project and task names; use an allowlist and send the rest as `other`.
- **Credentials:**
  - Action 23 runs as root every day (`runme.sh:2022`) with whatever is in the host's `hubs.toml`.
  - Discovery §2.9 says that file may hold .107's full hub secret, so the design widens how often it is used.
  - Mint a post-only token scoped to `telemetry:daily` before rollout.
- **A host can overwrite another host's report.**
  - The host name is self-declared (`TELEMETRY_HOST`, `--metadata host=`; `telemetry-rollup.sh:492`).
  - The fleet view keeps whichever report has the latest self-declared `generated_at` (`telemetry-fleet-view.sh:81-84`).
  - So a report stamped 2099 wins forever. Tie the host to the hub's record of who posted, and use the hub's timestamp.
- **Reports can drop out of view.** The view reads `--limit 5000` from cursor 0 (`telemetry-fleet-view.sh:44`). Once the topic holds more than 5,000 posts, the newest reports silently disappear.

## 5. Data quality

- **Rotation cuts days short, and nothing says so.**
  - On .107 the backup log spans about 18 h, and the live file grew 15 MB in 6.5 h, so the 2×32 MiB cap holds roughly 24-27 h.
  - The rollup at 01:30Z needs 25.5 h of history, and the attribution fields make rows longer.
  - No check compares the oldest kept row with the day's start (`telemetry-rollup.sh:308-337`), so a short day posts as low usage.
  - Fix: mark each day complete or not, or better, count hourly on the host.
- **Catch-up for days older than about one day** reads logs that have already rotated away and posts near-zero usage. It is flagged LATE, but a later sum would treat it as real. Post `usage: null` instead.
- **Two bugs in the append code break the "no undercount" promise:**
  - It makes two separate writes, the line and then `\n` (`audit_append.rs:36-37`), so concurrent writers can interleave and corrupt lines.
  - If two processes rotate at the same moment (`:27-33`), the second rename overwrites the backup and loses about 32 MiB.
- **Gaps on the remote hubs:**
  - Non-root agents point at the hub's directory; if the hub runs as root they likely can't write there, and the error is swallowed (`invocation_audit.rs:238,402`).
  - The root rollup reads only `/tmp/termlink-0` among the old fallback logs.
  - It runs from `/var/lib/termlink-telemetry`, which has no dispatch, verdict or applied files (`telemetry-rollup.sh:141-144`).
  - It also finds the runtime directory differently from the binary (`:120-123`), the trap the module doc warns about.
- **Polling by name:** `list` and `hub status` are often deliberate one-off calls (`:300-301`). Separate polling by call rate instead.
- **"A channel cannot report its own failures":** correct, and MISSING only partly answers it. Add an out-of-band pull of each host's posted-days file.
- **Two weeks** is enough for CLI vs MCP, and for cutting tools with zero use on complete data. It is not enough for worker fitness or rarely used tools. Require, for example, four weeks of zero use with complete data on every host before a "cut".

## 6. Top five changes before slice 4, by impact

1. **Fix the undercount paths:**
   - one write per line, and lock the rotation;
   - per-sink coverage and completeness flags;
   - count hourly on the host;
   - fix the root vs non-root split on the remote hubs.
2. **Stop filing non-Anthropic agents as `background`.** Detect their markers or have their launchers set class, kind and provider; add the `ps` cross-check; attribute `session-rpc` rows to the caller.
3. **Build the census from the harness side** (process scan plus harness logs), not from TermLink rows.
4. **Rework the outcome metrics:**
   - compute them over a trailing window instead of freezing daily snapshots;
   - add duration, tokens and cost, fallback, and finer task types;
   - record the resolved model ID and an explicit kind;
   - check result content for refusals;
   - break usage down by harness, model and kind.
5. **Trust:** a scoped post-only token, host tied to the sender, hub timestamps, and proper paging in the fleet view.

Drop proposal §5.8 (deriving provider from kind): opencode can front any provider.

## 7. Confidence and what I could not verify

| Answer | Confidence |
|---|---|
| 1 | high |
| 2 | high (medium on what other harnesses set in their environment) |
| 3 | high |
| 4 | high on overwrite and paging; medium on credential scope |
| 5 | high on the code defects; medium on volumes (one host, one snapshot) |
| 6 | medium (judgement) |

Not verified:
- which environment variables Codex, opencode and Antigravity set;
- whether the hub records who sent a `channel post`;
- the contents of `hubs.toml` on the remote hosts;
- whether the hub directory on .122, .121 and .132 is writable by non-root users;
- whether dispatch always writes `worker_kind.txt` for non-Claude workers;
- that `cli_verb_path` carries no arguments (assumed).
