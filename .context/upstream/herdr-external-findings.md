# Herdr — external evaluation findings (2026-08-15)

Companion to `.context/upstream/herdr-evaluation-recon-2026-08-15.md`. This file
covers the **external** half only: license, identity, socket API, maturity.
Our internal tmux surface is out of scope here (separate agent).

**Evidence convention used throughout.** Every claim is tagged:

- **[VERIFIED-SOURCE]** — read at a pinned tag/commit or an authoritative API
  endpoint, with the URL given. Verbatim where quoted.
- **[VERIFIED-PAGE]** — read from a live project-owned page (herdr.dev docs,
  GitHub UI) that is not pinned and may change.
- **[SECONDARY]** — a blog, aggregator, or search summary. Treated as a lead,
  never as proof.

A caveat that applies to every WebFetch result below: the fetch tool summarises
pages through a small model. Direct quotes are reliable; counts and enumerations
carry some transcription risk and are flagged where it matters.

---

## 1. LICENSE — RESOLVED. The project relicensed. Both sources were right.

**Verdict: `Apache-2.0` as of v0.8.0 (2026-08-03) and current `master`.
Everything at or before v0.7.5 (2026-07-21) is `AGPL-3.0-or-later` with a
commercial dual-license. Confidence: HIGH.**

The conflict in the recon file was not an error by either source — it was two
sources observed on opposite sides of a relicense that happened **2026-07-22**,
about three weeks before this evaluation.

### The commit trail

`https://github.com/ogulcancelik/herdr/commits/master/LICENSE` **[VERIFIED-PAGE]**
— exactly three commits have ever touched `LICENSE`:

| Date | Commit | Message |
|---|---|---|
| 2026-03-27 | `a57b972` | `initial release` |
| 2026-05-26 | `cfffe65` | `docs: clarify dual licensing` |
| 2026-07-22 | `cd5ea1b` | `chore: relicense herdr under apache-2.0` |

### Reads at pinned refs — this is the load-bearing evidence

**At the initial commit** — `raw.githubusercontent.com/ogulcancelik/herdr/a57b972/LICENSE`
**[VERIFIED-SOURCE]**, verbatim first lines:

```
GNU AFFERO GENERAL PUBLIC LICENSE
                       Version 3, 19 November 2007
```

**At tag `v0.7.5`** (2026-07-21, the last release before the relicense) —
`raw.githubusercontent.com/ogulcancelik/herdr/v0.7.5/LICENSE`
**[VERIFIED-SOURCE]**, verbatim first lines:

```
Herdr is dual-licensed:

1. Open source: GNU Affero General Public License v3.0 or later (AGPL-3.0-or-later).
   See the full text below.

2. Commercial: commercial licenses are available for organizations that cannot comply
   with AGPL.
```

This is the direct confirmation of the recon file's suspicion: the paid
commercial license was **real**, and it existed precisely because the open-source
side was AGPL. The recon's reasoning — "nobody sells an exception to Apache 2.0"
— was correct, and it correctly identified the AGPL reading as genuine for *some
version* of the project. That version was every version up to v0.7.5.

**At tag `v0.8.0`** (2026-08-03) —
`raw.githubusercontent.com/ogulcancelik/herdr/v0.8.0/LICENSE` **[VERIFIED-SOURCE]**:

```
                                 Apache License
                           Version 2.0, January 2004
                        http://www.apache.org/licenses/
```

**At current `master`** — same Apache 2.0 text **[VERIFIED-SOURCE]**. GitHub's
API reports `license.spdx_id: "Apache-2.0"` **[VERIFIED-SOURCE]**.

### What the relicense commit actually changed

`https://github.com/ogulcancelik/herdr/commit/cd5ea1b` **[VERIFIED-PAGE]** —
6 files: `Cargo.toml`, `LICENSE`, `README.md`, `docs/next/CHANGELOG.md`,
`docs/next/README.md`, `nix/package.nix`. The `Cargo.toml` license field moved
`AGPL-3.0-or-later` → `Apache-2.0`, and the README's dual-licensing/commercial
language was deleted outright. `CHANGELOG.md` for `[0.8.0]` records it in one
line **[VERIFIED-SOURCE]**: *"Relicensed Herdr from AGPL-3.0-or-later to
Apache-2.0."*

### Why the secondary sources still say AGPL

`lib.rs/crates/herdr` **[VERIFIED-PAGE]** reports `AGPL-3.0-or-later` — because
the only crate ever published to crates.io is **v0.1.0, 2026-03-27**, whose
manifest carried the AGPL field. The registry metadata is frozen at the March
snapshot and has not been republished. Every blog and aggregator citing AGPL +
commercial licensing is either reading that stale crate metadata or was written
before 2026-07-22. **They are not wrong; they are stale.** [SECONDARY]

### The residual risks — flagged, not resolved

1. **Relicensing AGPL → Apache requires consent from every copyright holder.**
   The commit message is a bare `chore:` line and explains nothing — no stated
   rationale, no CLA reference, no contributor-consent note **[VERIFIED-PAGE]**.
   The repo has 2,070 forks and 141+ open issues, so outside contributions are
   near-certain. I found **no CLA and no DCO evidence** either way — I did not
   read `CONTRIBUTING.md`, and that is the single highest-value follow-up if
   anyone intends to rely on the Apache grant. An incompletely-executed
   relicense is a real (if uncommon) legal exposure.
2. **Anything you already hold at ≤ v0.7.5 is AGPL.** If a herdr binary or
   source drop entered this project before 2026-08-03, it came under AGPL terms.
   Upgrading to v0.8.0+ is the clean path.
3. **A project that relicensed once can relicense again.** The move went
   *toward* permissive, which is the friendly direction, but 0.x + one relicense
   in 141 days is a governance signal worth weighing.

---

## 2. Identity and version conflicts — RESOLVED

**Canonical repo: `herdrdev/herdr`.** `api.github.com/repos/ogulcancelik/herdr`
returns `full_name: "herdrdev/herdr"`, `owner.login: "herdrdev"`,
`fork: false`, no `parent` **[VERIFIED-SOURCE]**. So `ogulcancelik/herdr` is not
a fork and not a different project — it is the **old path of the same repo**,
renamed/transferred from a personal account to an org, with GitHub serving its
permanent redirect. Both URLs return identical stars and identical tags. The
recon file's "fork? rename? different project?" is answered: **rename/transfer**.

**Current release: `v0.8.0`, 2026-08-03** **[VERIFIED-PAGE]**
(`/releases` and `/tags` agree; `CHANGELOG.md` latest entry is `[0.8.0] - 2026-08-03`
**[VERIFIED-SOURCE]**).

**The v0.4.0 figure appears to be a misread, not a conflict.** Two independent
fetches of the repo landing page reported latest release as `v.0.4.0` — note the
stray dot, which is not the repo's tag format (`v0.8.0`). One fetch explicitly
qualified it as *"(visible in video demo)"*. The README embeds a demo
video/GIF recorded at v0.4.0. I am **moderately confident** this is an artifact
of the summarising model reading a version string off the demo asset rather than
the release widget. I did not fully disprove it. Treat **v0.8.0 as actual** —
that is what the tags, releases page, and changelog all independently say.

**Stars: 29,247** — `stargazers_count` from the GitHub API **[VERIFIED-SOURCE]**.
The ~15k figure is a stale secondary snapshot. Also: 2,070 forks, 91 watchers.
Note the ratio — 29.2k stars against only 91 watchers is characteristic of a
viral trending spike rather than deep adoption. Stars are attention, not usage.

---

## 3. Socket API — what it can and cannot do

Source: `https://herdr.dev/docs/socket-api/` **[VERIFIED-PAGE]** (unpinned docs;
not a pinned spec file).

**Transport:** newline-delimited JSON over a Unix domain socket (named pipes on
Windows). Request-response with optional event streaming. There is a **protocol
version** for client/server compatibility; the docs advise *"Check the server
protocol with `ping` or `herdr status` before depending on new behavior. Handle
unknown fields gracefully."* **[VERIFIED-PAGE]** — note this is a compatibility
*practice*, not a stability *guarantee*.

Scored against the operations a session backend must support:

| Operation TermLink needs | Herdr | Method |
|---|---|---|
| Spawn a session | **YES** | `workspace.create`, `pane.split` |
| Inject / send input | **YES** | `pane.send_text`, `pane.send_keys`, `pane.send_input` |
| Capture output | **YES** | `pane.read` (sources: visible, recent, recent-unwrapped, detection) |
| Resize PTY | **YES** | `pane.resize` |
| List / query sessions | **YES** | `pane.list`, `pane.get`, `pane.current`, `workspace.list/get`, `session.snapshot` |
| Detect session death | **PARTIAL** | `events.subscribe` → `pane.exited` event; **payload undocumented** |
| **Retrieve exit code** | **NO** | *no method returns a process exit code or exit status* |
| **Deliver signals** | **NO** | *no signal API; only character-based `send_keys`* |

**The bottom two rows are the decision-critical finding.** They are exactly the
primitives TermLink's founding verb depends on:

- `termlink exec <s> '...' --json` returns `exit_code`, and
  `scripts/session-selftest.sh` (T-2485, the verb-4 prover) **asserts
  `exit_code 0`** in its EXEC stage. Herdr's socket API exposes no exit code, so
  that assertion could not be satisfied through it. The nearest substitute is
  `pane.process_info` — which returns *"the pane's shell pid, foreground process
  group id when available, and foreground processes with pid, name, argv/cmdline,
  and cwd when the platform exposes them"* **[VERIFIED-PAGE]** — liveness and
  identity, but no status. You would be reduced to scraping `$?` off the screen.
- `termlink signal` exists, and session-selftest's CLEANUP stage uses
  `signal TERM`. Herdr offers no signal delivery. `send_keys` with `C-c` reaches
  the *foreground* process via the tty line discipline only — it cannot target a
  pid, cannot reach a backgrounded or detached process, and cannot deliver
  anything but the terminal-generated signals.

Both gaps are architectural, not missing polish: herdr models a *pane a human
watches*, so it exposes what a screen shows. TermLink models a *command whose
result is consumed programmatically*, so it needs status.

**Also relevant:** open issue **#2805 "pane.read always returns revision: 0"**
**[VERIFIED-PAGE]** — an active correctness bug in the exact output-capture
primitive an integration would depend on.

**Agent state detection is heuristic screen-scraping, not a protocol.**
`https://herdr.dev/docs/agents/` **[VERIFIED-PAGE]**, verbatim: *"Herdr
identifies the foreground process and reads the live bottom-buffer screen
snapshot. It evaluates TOML manifests against that snapshot to classify `idle`,
`working`, and `blocked`."* Agents with lifecycle hooks installed report
authoritatively; everything else is pattern-matched off the rendered screen. The
docs concede the failure mode: *"Unusual new agent prompts may initially show as
`idle` instead of `blocked` until detection manifests are updated."*

This materially downgrades the recon file's verb-1 overlap. Herdr's "discover"
is inference from pixels; TermLink's `agent-presence` heartbeat + cv_index is a
protocol with an explicit liveness contract and a canary (T-2387) proving the
rail is armed. These are not two implementations of the same thing.

---

## 4. Maturity and risk

**Age: 141 days.** `created_at: 2026-03-27T17:54:33Z` **[VERIFIED-SOURCE]**.
`pushed_at: 2026-08-15T01:23:49Z` — active as of today.

**Release cadence: extremely fast.** `CHANGELOG.md` lists ~47 releases from
`[0.1.2] 2026-03-28` to `[0.8.0] 2026-08-03` **[VERIFIED-SOURCE]** — roughly one
release every three days across 141 days. Plus dated `preview-YYYY-MM-DD-<sha>`
tags between releases **[VERIFIED-PAGE]**.

**Breaking-change history — the strongest pre-stability signal.** Verbatim from
`CHANGELOG.md` **[VERIFIED-SOURCE]**:

- `[0.7.0]` — *"Bumped the client/server protocol version to 14 for `pane.move`
  compatibility."* and *"Public workspace, tab, and pane ids are now short stable
  handles such as `w1`, `w1:t1`, and `w1:p1`; closed tab and pane ids no longer
  retarget later resources."*
- `[0.6.0]` — *"The client/server protocol is now version 8."* and *"Removed the
  separate `keys.quit` binding."*
- `[0.5.0]` — *"herdr now defaults to a persistent server/client session model."*
- `[0.4.7]` — *"Socket API clients that match `result.type` exactly need to
  handle `workspace_created` and `tab_created`."*
- `[0.7.5]` — plugin install state moved from per-session to per-user.

**The protocol went from version 8 to version 14 inside a single minor release
step (0.6.0 → 0.7.0).** Six wire-protocol revisions in one cycle. Object
identity semantics changed in 0.7.0. The socket API — the exact surface an
integration binds to — has broken repeatedly and recently.

**Stability guarantees: none found.** The docs index has no page on API
stability, versioning policy, or deprecation **[VERIFIED-PAGE]**. The only
stability language anywhere is the socket-api page's "handle unknown fields
gracefully" advice, which is guidance to clients, not a commitment from the
project. **No page on licensing or pricing exists on the docs site either** —
consistent with the commercial license having been retired at v0.8.0.

**Issues: ~142 open** (GitHub UI) / `open_issues_count: 168` via API — the API
figure includes PRs, which reconciles with ~26 open PRs **[VERIFIED-SOURCE /
VERIFIED-PAGE]**. Character: overwhelmingly **bug reports about terminal
correctness**, not feature requests — mouse input dropped under ConPTY/OpenSSH,
CSI-u sequence fragments leaking on `Ctrl+<letter>`, raw OSC 10/11 color-query
responses printed to screen, panes stuck at an 80x24 floor when no client is
attached, all visible panes flickering to an older frame every ~12s, `pane.read`
returning `revision: 0`. This is a young terminal emulator still finding its
edges across platforms.

**Maintainer concentration: strongly suggestive of one person, not confirmed.**
The contributors graph failed to load on fetch, so I could not get commit counts
— **this is an unresolved gap, stated rather than guessed**. Circumstantial
evidence for a single maintainer: the repo originated on a personal account
(`ogulcancelik`), the relicense was authored by `ogulcancelik` alone, a
`SPONSORS.md` exists at root, and commercial licensing was routed to a single
address (`hey@herdr.dev`). Only 91 watchers against 29.2k stars.

**macOS: supported but not evidently first-class.** README lists
`brew install herdr` for macOS, a curl script for Linux, and PowerShell for
*"windows beta"* **[VERIFIED-SOURCE]** — so macOS is not flagged beta and reads
as a tier-1 target. But the *evidence* of macOS quality is thin: a search of
open issues for "macos" surfaced essentially one substantive hit ("herdr doesn't
work with devshell on macOS") **[VERIFIED-PAGE]**, and the visible bug traffic
is dominated by Linux, Windows/ConPTY/WSL2, and SSH-client scenarios. Low
macOS-specific issue volume is genuinely ambiguous — it can mean "solid" or it
can mean "few people run it there." **I cannot distinguish those two readings
from the outside, and I am not going to pick the convenient one.** For our
purposes the relevant asymmetry is that our own macOS CI (T-2692) is still
non-blocking, so we would be stacking an unmeasured dependency on an unmeasured
platform.

---

## 5. Conflicts I could not fully resolve

1. **v0.4.0 on the repo landing page.** Explained as a probable demo-asset
   misread (stray dot, one fetch attributed it to a video). Not disproven.
   Everything authoritative says v0.8.0.
2. **Maintainer count.** Contributors graph would not load. Single-maintainer is
   inferred from circumstantial evidence, not measured.
3. **Contributor consent for the relicense.** No CLA/DCO evidence found either
   way; `CONTRIBUTING.md` not read. This is the top follow-up if the Apache
   grant is to be relied on.
4. **macOS quality.** Low issue volume is ambiguous between "solid" and
   "unused". Undetermined from outside.
5. **`pane.exited` payload.** Documented as an event type with no payload spec.
   Whether it carries an exit status is unknown — this is the one place an exit
   code *might* surface, and it would need an empirical test against a running
   server to settle.

---

## 6. What this changes about the recon file's framing

The recon file gated everything on the license and warned that AGPL would make
most of the rest moot. **That gate is now open** — v0.8.0+ is Apache-2.0, which
imposes no reciprocal obligation on a shipped binary.

But the gate opening does not make the case. It just moves the decision onto the
technical and strategic merits, where the findings are less favourable than the
traction numbers suggest:

- The socket API **cannot supply exit codes or signals** — the two primitives
  our verb-4 prover asserts on. This is not a gap to be filled by a patch; it
  follows from herdr modelling a human-watched pane rather than a
  programmatically-consumed command.
- The wire protocol broke six times in one minor version and has no stability
  policy.
- The verb-1 overlap the recon flagged is weaker than it appeared: herdr's agent
  state is screen-scraped heuristics against TOML manifests, ours is a protocol.

The recon's anti-recommendation stands and is reinforced: 29.2k stars in 141
days is evidence of interest, not fitness for a load-bearing dependency under
the founding charter verb. The burden of proof remains on the replacement, and
on the two rows that read **NO** in the socket API table.
