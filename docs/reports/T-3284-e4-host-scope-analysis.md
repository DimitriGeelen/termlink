# T-3284 — E4 host scope: what E4 claims, where it can go wrong, and what to do

Research artifact for the inception T-3284 (operator request, 2026-09-30: "background,
implications, pros/cons, steelman, strawman, scoring against our framework directives, and
what is actually the exposure"). Every factual statement cites what it was measured from.
Where something is an inference, it is labelled as one.

## 1. Background: what E4 is and why it exists

- **arc-003 "reliable comms"** is a closed arc whose headline claim is *no silent loss of
  messages*. T-3066 made every closed arc bind its claim to an executable **prover**, so the
  claim is re-checked instead of trusted forever. arc-003's prover is
  `bash scripts/notify-rail-e2e.sh --stages '' --experiment e4`
  (`.context/arcs/reliable-comms.yaml:19`).
- **Who runs it:** `scripts/check-arc-claim-drift.sh`, a **FAIL-tier** member of our guard
  layer (`scripts/run-guard-layer.sh`). The guard layer runs in agent sessions on this host,
  and at the push (`doc-lint.yml`) and release (`release.yml`) gates. The release build
  depends on it passing.
- **What E4 asks:** "If a peer sends mail to this agent's identity, is any declared sidecar
  watching that mailbox?" Steps (`scripts/notify-rail-e2e.sh`, `experiment_e4`):
  1. resolve the identity: `TERMLINK_AGENT_ID=claude-termlink termlink agent identity --resolve --json`
  2. read `.context/cron/notify-sidecar-agents.conf` (declared `<agent> <fingerprint>` lines)
  3. PASS if the resolved fingerprint appears in any declared line; otherwise FAIL "IDENTITY SPLIT".
- **Today on this host:** PASS in 0.03 s. `claude-termlink` resolves to `6738c073bbcc587a`,
  which the conf declares (measured 2026-09-30).
- **T-3283 (already shipped)** fixed the classification: "could not look" (jq missing,
  timeout, non-zero exit, non-JSON) is now TOOLING (exit 2), not a failed claim.

## 2. What was found (measured, not assumed)

| # | Finding | Evidence |
|---|---|---|
| F1 | `resolve` **never fails for lack of an identity; it creates one.** With an empty `$HOME` it returned `{"action":"resolved","fingerprint":"8eba7886a7701de8","ok":true}`, rc 0 | probe, T-3283 |
| F2 | The file it creates is the agent's **signing key**: `load_identity_or_create()` → `$HOME/.termlink/identities/claude-termlink.key` (precedence: `TERMLINK_IDENTITY_FILE` > `TERMLINK_IDENTITY_DIR` > `TERMLINK_AGENT_ID` > host key) | `crates/termlink-cli/src/commands/identity.rs:57`, `crates/termlink-session/src/agent_identity.rs:200-220` |
| F3 | The sidecar conf is **deployment-specific**: three lines, all this host's fingerprints, and no host column | `.context/cron/notify-sidecar-agents.conf` |
| F4 | The conf's labels drift from reality: it maps `claude-termlink` → `d1993c2c…` (the shared host key), but `claude-termlink` actually resolves to `6738c073…`, which is listed under `claude-termlink-alt`. E4 still passes because it matches **any** declared line, not the agent's own | conf and live resolve |
| F5 | Key-minting by checks and probes is an existing pattern: this host has 20 per-agent keys, including `acp-probe`, `arc004-probe`, `demo-c7`, `e2e-worker` | `ls ~/.termlink/identities/` |
| F6 | No workflow that runs the guard layer puts `termlink` on the PATH (`doc-lint.yml`, `release.yml`: no `cargo install`, no `GITHUB_PATH`). `install-check.yml` installs it but runs no guard layer | the workflow files |
| F7 | Therefore at the push and release gates E4 exits 2 (binary missing), which `check-arc-claim-drift` turns into **SKIP(CI)**. **arc-003's claim is never actually verified at those gates**; it is verified only where a live binary and identity exist | F6 + `check-arc-claim-drift.sh:199-205`. That the runner lacked the binary is inferred from the workflow files; the job logs are no longer retrievable |

## 3. The exposure: what can actually go wrong, and where

| Scenario | What happens today | Likelihood | Consequence |
|---|---|---|---|
| **E1. This host, normal state** (key present, sidecars declared) | PASS | current state | none |
| **E2. This host, `claude-termlink.key` missing** (deleted, a re-image, a restore from a backup without `~/.termlink/identities/`, key rotation done by moving the file) | E4 **silently mints a new signing key** for `claude-termlink`, then reports IDENTITY SPLIT. From then on the live agent signs with the new fingerprint, and peers addressing the old one hit a mailbox nobody watches. **The check causes the silent loss it is meant to detect**, and its FAIL message blames the deployment | low | **high**: message routing for the main agent silently changes. That is the arc-003 failure mode itself, and it breaks Directive 2 |
| **E3. Another machine running this repo's guard layer with termlink installed** (a new dev host, a container, a peer project host) | mints a key under that machine's `$HOME`, then FAIL "IDENTITY SPLIT" → `check-arc-claim-drift` FAILs, and if that machine gates a release, it blocks | no known instance today (the fleet hosts run the termlink binary, not this repo's guard layer; not verified host by host) | medium: a false failure plus a stray key; a real release block if it gates |
| **E4. CI gates as they are** | binary missing → exit 2 → SKIP(CI) | every push and release | a coverage gap, not an error: the claim is unverified there, and the SKIP line says so |
| **E5. CI gate if termlink ever lands on the PATH** (e.g. someone adds `cargo install` for another check) | becomes E3 on the runner: mint + FAIL → the release is blocked | low, but a one-line workflow change away | release blocked for a non-problem |

**Net exposure:** the dangerous case is **E2 on this host**, not a remote machine. A verification
check has a write path into the very identity it verifies. E3/E5 are false alarms, and E4 is an
honestly reported coverage gap.

## 4. Options

- **A. Status quo.** Leave E4 as T-3283 left it.
- **B. Host allowlist.** E4 asserts only on hosts named in the conf (add a host/fqdn marker);
  elsewhere it reports "not this deployment" (exit 2 → skip).
- **C. Read-only identity probe.** E4 must never create a key. Add `--no-create` to
  `termlink agent identity --resolve` (same resolver, same precedence, no create).
  "No key for this agent here" → exit 2 "nothing to verify on this host: `claude-termlink` has
  no identity here, so it has no mailbox to lose". An existing key is then checked as today.
- **D. Move E4 out of the release gate.** Run it only as a host canary on this machine; bind
  arc-003 to a prover that CI can actually run.
- **C+F4.** C, plus correcting the conf labels so each agent line names that agent's real
  fingerprint. Optionally tighten E4 to check the agent's OWN line, not any line.

## 5. Steelman and strawman

**A — status quo**
- *Steelman:* nothing has gone wrong. E4 passes here, CI skips honestly, and T-3283 already
  removed the false-FAIL causes. Doing nothing costs nothing and adds no code.
- *Strawman:* "it's fine because it hasn't broken yet", which is exactly how arc-003's
  original claim went stale (T-3066's origin). E2 is a latent silent-identity change in the
  main agent.

**B — host allowlist**
- *Steelman:* makes the claim's scope explicit ("this is a claim about the .107 deployment"),
  so other machines stop false-failing. Simple to reason about.
- *Strawman:* a hard-coded host list is a portability smell (Directive 4) and rots when hosts
  are renamed or re-IPed. **It does nothing about E2**: on the declared host, a missing key is
  still minted silently.

**C — read-only probe (`--no-create`)**
- *Steelman:* it removes the root defect. A check no longer writes the identity it checks, so
  E2 disappears. It scopes itself without any host list: a machine where the agent has no key
  has no mailbox to lose, so "nothing to verify" is literally true. Uses the one canonical
  resolver, with no re-implemented precedence to drift. Small, testable Rust change plus one
  prover line.
- *Strawman:* it changes the product CLI for a test's sake. A reviewer could say "just don't
  run E4 elsewhere". And on this host a *missing* key now reads as exit 2 "nothing to verify"
  instead of an alarm. That could hide a real deletion unless the message and the host canary
  make it loud (see risks).

**D — move out of the release gate**
- *Steelman:* the release gate should contain checks CI can actually execute. E4 needs a live
  host, so it belongs to the host canary tier. It stops pretending: today the gate only ever
  SKIPs it (F7).
- *Strawman:* it weakens the one structural re-check of a closed arc's claim, reversing
  T-3066's intent. It doesn't fix E2 either: the host canary would still mint.

## 6. Scoring against the four constitutional directives

Scale 0–3 (3 = strongly serves the directive). The scores are judgements, justified in one line
each.

| Directive | A status quo | B host list | C read-only probe | D out of gate |
|---|---|---|---|---|
| **1 Antifragility** (stress → learning, not damage) | 1: a missing key turns into a changed identity, damage not a signal | 1: same on the declared host | **3**: a missing key becomes a named, visible condition, and nothing is mutated | 1: moves the problem, keeps the mint |
| **2 Reliability** (predictable, observable, no silent failures) | 0: E2 is a silent state change *by a check* | 1: removes false FAILs elsewhere; E2 remains | **3**: no silent writes; every outcome is named (PASS / FAIL split / no key here) | 1: the gate stops silently SKIPping, but the canary still mints |
| **3 Usability** (sensible defaults, actionable errors) | 1: a FAIL "IDENTITY SPLIT" on a fresh host points the operator at the wrong cause | 2: clear "not this deployment" | **3**: "claude-termlink has no identity on this host — nothing to verify" is exactly actionable | 2: simpler gate, but one more tier to know about |
| **4 Portability** (no environment lock-in) | 1: a non-.107 host false-FAILs | 0: hard-codes hosts | **3**: works on any host without configuration | 2: gate becomes host-agnostic; claim becomes host-only |
| **Total (of 12)** | **3** | **4** | **12** | **6** |

## 7. Risks of the recommended option (C) and their mitigation

1. **A real key deletion on this host would read as "nothing to verify" (exit 2) instead of a
   failure.** Mitigations:
   - on this host exit 2 is **not** skipped: `check-arc-claim-drift` only skips exit 2 under
     `CI`, so it still shows as a finding here;
   - the message names the missing key path;
   - optionally the conf can declare "this agent must have a key here" for the host where
     sidecars run. That is the one place a host-specific assertion is legitimate. It is data,
     not a hard-coded host list.
2. **A product CLI change** (`--no-create`). It is additive: `resolve` is unchanged without the
   flag, so it is a backward-compatible parity item. MCP parity: the census allowlist lists
   MCP-only gaps; this is a CLI flag, not a tool.
3. **F4 (the label drift)** is independent and cheap: correct the conf, and optionally make
   E4 match the agent's own line. It should be a separate task.

## 8. Recommendation

**GO on C**, as two build tasks:
1. `termlink agent identity --resolve --no-create`: resolve with the canonical precedence, and
   exit with a distinct code and message when no key exists, **without creating one**. Tests:
   an existing key resolves; a missing key → the distinct code and **no file written**
   (asserted on disk).
2. E4 uses `--no-create`: no key → exit 2 "no identity for claude-termlink on this host —
   nothing to verify (missing: <path>)". A fixture pins that no key file is created, plus a
   mutant that re-enables creation.

**Separately (small):** fix the conf label drift (F4), and state the CI coverage gap (F7) in
the arc-003 record rather than letting SKIP(CI) read as coverage.

**Not recommended:** B (hard-coded hosts, doesn't fix E2) and D (weakens T-3066, doesn't fix E2).

## Dialogue log

- 2026-09-30: the operator challenged the GitHub framing ("which worker?", "where did it
  run?"). The R9 worker ran on this host (`termlink dispatch --backend background`); its
  "runner has no termlink on PATH" was an unverified inference, which I carried forward. It
  was corrected, and the scenario was re-derived from the code and measurements above.
- 2026-09-30: operator: "it's still a valid error to react on". T-3283 shipped the
  classification fix. This inception covers the remaining question (host scope and the mint
  side effect).
- 2026-09-30: operator requested this analysis: background, implications, pros/cons,
  steelman/strawman, directive scoring, exposure.
