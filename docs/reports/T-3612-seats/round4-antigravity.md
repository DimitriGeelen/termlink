<!-- T-3612 round 4, seat antigravity, source /tmp/tl-dispatch/t3612-r4-antigravity-f1401bbff53e/result.md, exit 0 -->
# Independent Review: TermLink Purpose, Objectives and Value Drivers (Round 4 — `T-3612` v0.2)

## Section Verdicts

| Section | Verdict | Summary |
|---|---|---|
| **1. Purpose** | **change** | Choose the main sentence, add explicit wake + attribution, and keep the synced charter anchor to one sentence so `check-charter-sentence-drift.sh` does not fail-closed. |
| **2. Product objectives (`P-1`..`P-7`)** | **change** | Keep `P-4`; revise `P-1`, `P-2`, `P-3`, `P-5`, `P-6`, `P-7` to remove a dev-layer script leak, restore dropped `O-2` measures, and separate `today` from `not yet` honestly. |
| **3. Product value drivers** | **change** | Weights and axes are sound, but reusing `id: F-RECALL` collides with a hardcoded AEF scorer in `estimator.py`, and new free drivers require declarative `scoring:` blocks. |
| **4. Non-goals (1–6)** | **change** | Add cross-hub directory card exchange (`OD-11`) to Non-goal 1; six existing code/state violations are documented below. |
| **5. Development-project set (`DP-1`..`DP-5`)** | **change** | Move `check-unacted-mail.sh` from `P-1` to `DP-2`, and fix the `DP-1..DP-4` omission of `DP-5` in §6.2. |
| **6. What this replaces** | **change** | Account for release-gate extractor failure, BVP estimator handler dispatch, and `T-3371` `objectives.yaml` verification checks. |
| **7. Open points (1–5)** | **accept** | Concrete recommendations provided for all five decisions below. |

---

## 1. Purpose (Section 1) — Verdict: `change`

1. **[F-1] Main sentence vs. alternative (`docs/reports/T-3612-round4-draft.md:21-25,41-43`).** The **main sentence** is better than the terminals-first alternative because it leads with the operator's Q1 ruling (`docs/reports/T-3612-purpose-rederivation.md:52-59`): end-to-end delivery truth to a live agent that can act, stage visibility (`T-3618`), and deliverability routing over orchestration policy (Q2, `:60-78`). However, the main sentence omits two load-bearing capabilities present in the code and Q1/Q5: **waking** the agent (`crates/termlink-session/src/pty.rs`, `crates/termlink-sidecar/`) and **attributable identity** (`crates/termlink-bus/src/ signing`, `T-3486`).
2. **[F-2] Single-sentence sync constraint (`scripts/check-charter-sentence-drift.sh:78,96-108`).** The release-gate drift check extracts a **single** sentence between `ANCHOR` and a terminal word across `docs/CHARTER.md`, `README.md`, and `docs/ARCHITECTURE.md`. Draft §1 proposes two sentences (`T-3612-round4-draft.md:21-25`). Keep the first sentence as the synced canonical sentence, and place the second sentence (`"It is the base layer..."`) in the surrounding charter paragraph.
   - **Replacement wording (synced sentence):**
     > *TermLink gets every message and every unit of handed-off work to a live agent that can act on it — waking or driving its terminal across one or many machines — while recording who acted and showing where every item is until it is handled.*
   - **Scope sentence (immediately following in `docs/CHARTER.md`):**
     > *It is the base layer: it resolves the recipient and the path to reach it, and leaves what work to run and who should do it to the framework above.*

---

## 2. Objectives `P-1` to `P-7` (Section 2) — Verdict: `change`

3. **[F-3] `P-1` (Delivered means an agent can act on it) — CHANGE (`T-3612-round4-draft.md:54-60`).**
   - *Layer leak:* `scripts/check-unacted-mail.sh` (`:59`) shells out to `$ROOT/.agentic-framework/bin/fw sidecar` (`scripts/check-unacted-mail.sh:56,68-71`) and reads `.107` shared-root DM conventions (`:83-88`). That is a dev-fleet script (`DP-2`), not a product binary measure.
   - *Missing `O-2` measures:* Merging `O-2` into `P-1` dropped two operator-approved `O-2` guarantees from `.tasks/active/T-3371-define-project-objectives.md:150`: **retention-gap notice** (`T-3304` IW-3, `commit 44f9ee033`) and **hub responsiveness under load** (`T-3488`: no `create`/`post` > 5s below 90% disk), plus unclosed restart-safe post dedupe (`G-088`, `README.md:61-64`).
   - **Replacement wording:** Keep the `P-1` statement and add: *"Readers whose cursor fell behind retention receive an explicit gap notice, and posts stay responsive under load."* **Measure:** *Today:* `scripts/comms-selftest.sh` passes, retention-gap signals fire on truncated reads (`44f9ee033`), and dead-letter / unconfirmed-delivery canaries are empty. *Not yet:* sender-visible `posted → fetched → queued → injected → acked` stage chain on every rail (`arc-011` PH-2..PH-4), restart-safe post dedupe (`G-088`), `channel.post`/`create` ≤ 5s below 90% disk (`T-3488`), and native `termlink` CLI/MCP unacted-mail query.

4. **[F-4] `P-2` (It finds the way to the recipient) — CHANGE (`T-3612-round4-draft.md:62-68`).**
   - *Evidence gap:* `crates/termlink-sidecar/src/book.rs:5-9,73-96` currently resolves only intra-project peer names from a static operator-written file (`"PH-2 sends stay inside the project"`, `:87`) and errors if `main` is self (`:77`). Five-level resolution, cross-hub directory cards (`OD-11`), and the readiness-gated fenced `main` lease (`.tasks/active/T-3359-hub-pinned-main-alias-live-session-binding-with-co.md:175-185`) are not yet built.
   - **Replacement wording:** Keep the statement; revise **Measure:** *Today:* static intra-project address book and pinned `main` alias (`crates/termlink-sidecar/src/book.rs`). *Next:* pinned-`main` liveness visibility (`T-3619`). *Not yet:* five-level hub/host/project/agent/session directory resolution (`OD-10`/`OD-11`), readiness-gated `main` lease (`T-3359`), R-68 takeover tests (`T-3360`), and hub-brokered direct circuits (`OD-1`).

5. **[F-5] `P-3` (Handed-off work is done once and recorded once) — CHANGE (`T-3612-round4-draft.md:70-77`).**
   - *Evidence gap:* Naming internal consumers (`AEF and ring20`, `:71`) couples the product charter to specific callers. In code, `crates/termlink-bus/src/worker_contract.rs` implements the state machine and `--worker-binding` (`commit c677ef64f`), while CLI/MCP wiring and shared conformance vectors (`.tasks/active/T-3504-termlink-cli-and-mcp-tools-for-the-dispatch-contra.md:7`) are still `captured`. Keep per-claim isolated workspace primitives (`termlink agent checkout|commit|publish`, `docs/reports/T-3469-agent-launch-surface-design.md:47-51`) here.
   - **Replacement wording:** *"One authoritative, CAS-fenced record per unit of work (`wc/0.3`), read identically by the hub and any consuming orchestrator. Handed off without races, executed in an isolated per-claim checkout when requested, and recovered when its worker dies. Not external side-effect exactly-once."* **Measure:** *Today:* `worker_contract.rs` CAS transitions, `--worker-binding`, substrate-smoke and stuck-claims canaries. *Not yet:* CLI/MCP dispatch-contract tools and conformance vectors (`T-3504`), and `termlink agent checkout|commit|publish` (`T-3470`/`T-3471`).

6. **[F-6] `P-4` (Terminals can be reached, driven and cleaned up) — KEEP (`T-3612-round4-draft.md:79-83`).** Outcome-shaped and backed by `crates/termlink-session/` and `scripts/session-selftest.sh`.

7. **[F-7] `P-5` (Every act is attributable and authorised) — CHANGE (`T-3612-round4-draft.md:85-90`).**
   - *Overclaim in `today`:* Draft line 87 claims *"today: signatures are verified on post and on read; there is a per-agent key."* In fact, `.tasks/active/T-3615-mcp-tool-usage-telemetry-attributed-call-counter-f.md:378,443-447` and `.tasks/active/T-3618-receiver-side-delivery-truth-session-start-mail-sw.md:84-86` document that co-resident projects on `.107` still share `uid 0` and a shared host root key, and remote nodes (`.121`, `.122`, `.132`) still hold `.107`'s full `hub.secret` pending `T-3543`.
   - **Replacement wording:** Keep the statement; revise **Measure:** *Today:* Ed25519 envelope signing/verification (`T-3486`), fail-closed key check (`T-3467`), and signed PTY inject (`T-3493`). *In flight:* replacing remote `hub.secret` copies with scoped Interact tokens (`T-3543`/`T-3615`). *Not yet:* per-agent/per-topic authorisation (`T-2422`), per-project Unix user isolation (`T-3553`), and per-peer public-key allow-list (`T-3423`).

8. **[F-8] `P-6` (What is released installs and runs) — CHANGE (`T-3612-round4-draft.md:92-95`).**
   - *Evidence gap:* `.github/workflows/release.yml:62-85,113-136` runs Linux/macOS tests and `--gate release`, and `scripts/check-release-publication-freshness.sh` checks asset publication, but nothing tests clean-host `install.sh`, upgrade, or rollback (`GP-12` in `docs/reports/T-3344-step1-rulings-summary.md:177`).
   - **Replacement wording:** Keep the statement; revise **Measure:** *Today:* `release.yml` Linux/macOS test + `--gate release` jobs and `release-publication` canary. *Not yet:* automated clean-host install, upgrade, and rollback smoke test.

9. **[F-9] `P-7` (The surface is proportionate) — CHANGE (`T-3612-round4-draft.md:97-102`).**
   - *Missing measure:* Name `scripts/check-charter-drift-freshness.sh` (which already tracks off-charter MCP tools against `.context/checks/charter-drift-allowlist`) and include MCP description token budget (`arc-005`).
   - **Replacement wording:** *"Every CLI verb and MCP tool has a caller or a coordination reason, and MCP tool descriptions stay within a declared token budget; what has neither is removed on a schedule."* **Measure:** *Today:* `scripts/check-charter-drift-freshness.sh` off-charter tool count. *Not yet:* attributed fleet usage telemetry (`T-3615`) and post-telemetry removal target.

---

## 3. Value Drivers (Section 3) — Verdict: `change`

10. **[F-10] Weights and `D1..D4` orthogonality (`T-3612-round4-draft.md:108-127`, `policy/bvp-scoring-rubric.md:29-170`).**
    - Promoting `D2 Reliability` (`7 → 10`) and `D3 Usability` (`5 → 7`) while keeping `D4 Portability` (`3`) matches the 4-seat synthesis (`docs/reports/T-3612-synthesis-round2.md:139-150`).
    - `F-TRUST` (cryptographic identity, least-privilege auth, OS isolation) and `F-SURFACE` (pruning dead/off-charter surface and context tax) represent axes that `D1` (stress-hardening), `D2` (no silent failure), `D3` (ergonomics), and `D4` (portability) do not mean.
    - Dropping `F-ORCH` fits Q2 (`T-3612-purpose-rederivation.md:60-66`), provided `D2`'s note explicitly covers `P-2` routing and `P-3` hand-off primitives so `T-3504`/`T-3470` do not lose BVP priority.

11. **[F-11] Critical `F-RECALL` ID collision & Q3 boundary (`T-3612-round4-draft.md:118-125`).**
    - *Mechanical collision:* In `.agentic-framework/agents/termlink/bvp-estimator/estimator.py:2631,2761-2771`, BVP scoring dispatches in strict order: **`handler -> alias -> declarative spec -> unscored`**. Because `"F-RECALL": score_f_recall` is hardcoded in `_handler_table()` (`:2631`), keeping `id: F-RECALL` executes `score_f_recall()` (`:525-584`), which greps for `fw recall`, `CLAUDE.md`, `learnings.yaml`, and `qdrant`, **ignoring any declarative `scoring:` block** (warned in `.agentic-framework/policy/value-drivers.yaml:375-378`). Similarly, `.agentic-framework/agents/audit/audit.sh:2078` runs a hardcoded `recall_signals()` `retire_when` check for `F-RECALL`.
    - *Q3 boundary (`T-3304:179-197`):* Draft line 119 says *"and the record hands what must be kept to a long-term store."* `T-3304` IW-1 ruled that long-term records live in git/files/DB, **not** that the hub builds an automated archival hand-off pipeline. Also, `T-3304` IW-3 approved single-topic cursor + sender filter + hub receive-time window reads, not cross-topic indexing.
    - **Replacement wording (rename ID to `F-CONV-RECALL`):** *"An agent can recover what was said and decided in a conversation topic within the retention window — via cursor, retention-gap notices, page-end reasons, and hub receive-time range reads (`T-3304` IW-3) — without silent skips or full-topic client downloads."*

---

## 4. Non-Goals (Section 4) — Verdict: `change`

12. **[F-12] Non-goal 1 omits `OD-11` directory card exchange (`T-3612-round4-draft.md:134-135`).**
    - `docs/reports/T-3344-step1-rulings-summary.md:104-107` ruled: *"hubs exchange cards, never messages (federation still out)."*
    - **Replacement wording for Non-goal 1:** *"No federation of topic messages. Hubs never relay each other's topic posts; hubs may exchange signed project directory cards (`OD-11`) and broker short-lived credentials for direct sidecar-to-sidecar circuits while remaining the fallback path (`OD-1`)."*

13. **[F-13] Existing code and state violations of Non-goals 1–6:**
    - *Non-goal 1 (Spoke-to-spoke circuits vs. strict-star tripwires):* `README.md:68`, `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs:1-36`, and `scripts/check-strict-star.sh:8` enforce *"Strict star; spokes never connect to one another"*, which conflicts with `OD-1` sidecar-to-sidecar circuits and `arc-003` direct host transport.
    - *Non-goal 2 (Not an archival database):* 75 of 111 hub topics currently have `forever` retention without owners (`.tasks/active/T-3304-bounded-hub-retention-and-the-memory-vs-messaging-.md:181,189`; `T-3314` still `captured`).
    - *Non-goal 3 (Not a social/engagement platform):* `crates/termlink-mcp/src/tools.rs` still ships reactions, stars, pins, typing indicators (`termlink_agent_typers`, `:19997`), polls (`termlink_channel_poll_start`, `:22433`), emoji stats (`:21391`), 28 allowlisted social-analytics tools (`termlink_agent_top_repliers`, `:1307`, `.context/checks/charter-drift-allowlist`), and 40+ deprecated tools (`T-3612-purpose-rederivation.md:103`).
    - *Non-goal 4 (No planning/task policy):* `crates/termlink-hub/src/router.rs:78-80` still compiles `orchestrator.route`, `orchestrator.bypass_status`, and `orchestrator.bypass_invalidate`.
    - *Non-goal 5 (Attributable/authorised fleet):* Co-resident agents on `.107` still run as shared `uid 0` with a shared root key, and `.121`/`.122`/`.132` hold `.107`'s full `hub.secret` (`T-3615:378,443`, `T-3618:85`, `T-3553:120`).
    - *Non-goal 6 (Not a governance/host-ops platform):* `runme.sh` still embeds AEF actions (`11`, `11b`, `11c`, `13`, `16`) and `ring20` action `22` (`.tasks/active/T-3612-derive-termlinks-value-drivers-and-objectives-from.md:181-182`).

---

## 5. The Development Project's Set (Section 5) — Verdict: `change`

14. **[F-14] Layer separation fixes (`T-3612-round4-draft.md:59,154,165-167,174`).**
    - Move `scripts/check-unacted-mail.sh` from `P-1` (`:59`) into `DP-2` (`:154`).
    - Fix §6.2 (`:174`), which says *"a `project_objectives:` list for `DP-1..DP-4`"* — v0.2 added `DP-5` (`:161,194`), so §6.2 must read `DP-1..DP-5`.

---

## 6. Open Points (Section 7) — Verdict: `accept`

15. **[F-15] Recommendations on Open Points 1–5 (`T-3612-round4-draft.md:180-190`):**
    1. **Purpose sentence:** Choose the **main sentence**, edited per **[F-1]/[F-2]** to include waking and attribution in a single synced sentence. *Reason:* Leads with Q1's delivery-truth requirement while preserving `check-charter-sentence-drift.sh` compatibility.
    2. **`P-3` per-worker workspace:** **Keep the workspace mechanism in TermLink (`P-3`); leave isolation policy to AEF.** *Reason:* Operator already ruled GO on `B` in `.tasks/active/T-3469-agent-launch-surface-clean-env-workspace-checkout-.md:380-395` (`termlink agent checkout|commit|publish` fenced by `channel.renew`), and `termlink dispatch --isolate` already ships in `crates/termlink-cli/src/commands/dispatch.rs:31,71-93`.
    3. **`D1` weight & `DP-3` canary cap:** Set product **`D1 = 6`** (below `D2=10`, `D3=7`, `F-TRUST=7`, high enough to value fault-injection provers like `T-3360`), and cap `DP-3` at **35 cron canaries** (down from 45 `.crontab` files today) and **75 source guards** (down from 98 `scripts/check-*.sh`) under a strict one-in, one-out rule.
    4. **Per-layer driver tooling (§6.3):** **Do both now:** write the dev-project set to `policy/value-drivers-dev.yaml` immediately (for decision briefs, as in `docs/reports/T-3469-iw3-decision-brief.md:87-115`) and file an AEF feature request (`G-062`) for task-frontmatter-routed BVP scoring.
    5. **Product recall (`F-CONV-RECALL`):** Narrow wording per **[F-11]**, rename ID to `F-CONV-RECALL`, and set **`retire_when`**: *"Retire when `arc-012` closes: retention-gap notices, page-end reasons, hub receive-time range reads (`T-3304` IW-3), topic ownership/idle/unread flags (`IW-4`), and the 75 forever-topic review (`T-3314`) are live on all fleet hubs."*

---

## 7. What Would Break If Adopted As Written? (Section 6) — Verdict: `change`

16. **[F-16] Four concrete breakages on adoption:**
    1. **Release & CI guard failure (`scripts/check-charter-sentence-drift.sh:78,104-107`):** Hardcodes `ANCHOR="TermLink is a hub-mediated, durable append-log message bus"` and truncates at `*machines*`. Adopting Draft §1 without updating `ANCHOR`, the terminal word match, and `scripts/test-charter-sentence-drift.sh:22` returns `__NOMATCH__` (`exit 2`, fail-closed tooling error) in `scripts/run-guard-layer.sh --gate release` (`.github/workflows/release.yml:85`).
    2. **BVP estimator & audit breakage (`estimator.py:2631,2782`, `lib/bvp-scorability.sh:81-83`):**
       - `F-RECALL` is hijacked by `score_f_recall()` (`estimator.py:2631`) and `recall_signals()` (`audit.sh:2078`).
       - `F-TRUST` and `F-SURFACE` have no handler in `estimator.py:_handler_table()` (`:2618-2647`) and no declarative `scoring:` block (`kind: signals`, `T-3428`) in Draft §3; `estimator.py:2782` will drop them as `unscored` (0 weight in all rankings) and `audit.sh:5022` will emit a persistent `WARN`.
    3. **`objectives.yaml` schema & arc resolution (`T-3371:100-102`):** `T-3371` checks `grep -qi 'authored intent; progress is derived' .context/project/objectives.yaml` and requires `id`, `text`, `measure` on `d['objectives']`. Because `O-5` splits across `P-6` and `DP-2` and `O-6` moves to `DP-1`, arcs with `supports: [O-5, O-6]` will fail resolution unless `supersedes:` spans both `objectives:` and `project_objectives:` or arc YAML files are updated directly.
    4. **Tripwire tests (`crates/termlink-hub/tests/no_federation_tripwire.rs`, `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs`, `scripts/check-strict-star.sh`):** Will fail when `OD-1` sidecar-to-sidecar circuits and `OD-11` cross-hub card exchange land unless their allowlists/assertions are updated.

---

## 8. Three Most Important Changes, Ranked

17. **[F-17] Ranked top 3 changes:**
    1. **Rename product `F-RECALL` to `F-CONV-RECALL` and ship declarative `scoring:` blocks (`kind: signals`) for `F-TRUST`, `F-SURFACE`, and `F-CONV-RECALL`** so `estimator.py:2631,2782` and `audit.sh:2078,5022` score them instead of running AEF's memory handler or dropping them as `unscored`.
    2. **Keep the synced charter purpose to a single sentence (adding wake + attribution) and update `scripts/check-charter-sentence-drift.sh` + `scripts/test-charter-sentence-drift.sh` in the same commit** so `--gate release` does not fail-closed with exit code 2.
    3. **Clean up `P-1`..`P-7` measures and Non-goal 1:** move `check-unacted-mail.sh` to `DP-2`, restore `O-2`'s retention-gap and ≤5s load measures to `P-1`, separate `today` vs. `not yet` in `P-2`/`P-5`/`P-6`, add `OD-11` card exchange to Non-goal 1, and include `DP-5` in §6.2.

---

## Confidence & What Could Not Be Read

- **Confidence:** **High.**
- **What could not be read directly:** The review workspace was exported via `git archive -- . ':(exclude).context'`, which excluded `.git/` and `.context/` (`.context/project/objectives.yaml`, `.context/arcs/*.yaml`). Their exact contents, schema checks, and 2026-10-21 operator edits were verified through `.tasks/active/T-3371-define-project-objectives.md`, `.tasks/active/T-3612-*.md`, `docs/reports/T-3612-*.md`, and `scripts/check-arc-*.sh`. Live hub runtime state was not queried per read-only rules.
