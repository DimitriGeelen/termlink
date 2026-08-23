# Herdr adoption evaluation — synthesis and recommendation (2026-08-15)

**Recommendation: NO** on adopting herdr as TermLink's session layer.
**Optional cheap YES** available on adding herdr as a 4th spawn-launcher (~0.5 day)
if anyone actually wants it — but nobody has asked for that, and it is not what
the question meant.

Sources: `herdr-internal-tmux-surface.md`, `herdr-external-findings.md`,
`herdr-evaluation-recon-2026-08-15.md`. Both investigations line-cite their
claims; caveats on coverage are carried through below rather than dropped.

---

## 1. The question's premise was false

Asked: *"will we benefit from adopting herdr, replacing our current tmux solution?"*

**There is no tmux solution to replace.** TermLink owns its PTYs natively —
`libc::openpty()`/`fork()`/`execvp()` at `crates/termlink-session/src/pty.rs:103–120`.
No `tmux` module, no `backend` module among the 30 in `lib.rs`. tmux is one of
three interchangeable *process launchers* that start a detached shell which then
runs `termlink register`; `spawn_via_tmux` is **12 lines**
(`commands/execution.rs:528–539`) behind a single dispatch site. It is never read
back — no `capture-pane`, no `send-keys`, no `list-sessions`. The handle is
write-only and readiness comes from polling TermLink's own registry.
`commands/pty.rs` — 1263 lines of inject/output/resize/attach/stream/mirror —
contains **zero** tmux references.

So the question splits in two:

| Interpretation | Meaning | Cost |
|---|---|---|
| Add a 4th launcher | herdr joins Terminal/Tmux/Background as a way to start a detached shell | **~0.5 day** |
| Replace the PTY layer | what herdr actually *is* | **weeks–months + a charter decision** |

Everything below concerns the second, because that is what "adopting herdr"
would mean in practice.

## 2. The licence gate is open — and it barely matters

**Resolved, HIGH confidence, verified by reading `LICENSE` at pinned refs:**

| Ref | Licence |
|---|---|
| `a57b972` (initial, 2026-03-27) | AGPL-3.0 |
| tag `v0.7.5` (2026-07-21) | AGPL-3.0-or-later **+ paid commercial** dual-licence |
| tag `v0.8.0` (2026-08-03), `master` | **Apache-2.0** |

The project **relicensed on 2026-07-22** (`cd5ea1b`, recorded in `CHANGELOG.md`
under `[0.8.0]`). Both conflicting sources were correct, observed on opposite
sides of that event — a reminder that "sources disagree" sometimes means "the
world changed", not "someone is wrong". The recon's reasoning that a paid
commercial exception proved a genuine AGPL reading was right; that licence was
real and has been retired.

**Residual risk:** the relicense landed as a bare `chore:` commit with no
rationale, no CLA/DCO evidence found, against 2,070 forks. AGPL→Apache requires
every copyright holder's consent. If we ever *did* depend on the Apache grant,
that provenance needs checking. It does not need checking now, because —

**the licence only ever mattered for the expensive path.** A launcher is a
subprocess boundary and is licence-safe under either licence. Linking is where
copyleft would have bitten, and we are not going to link, for the reasons in §3.

## 3. The decisive finding: two architectural NOs

The socket API cannot do two things TermLink's charter verb 4 depends on:

- **It cannot retrieve exit codes.** No method returns process exit status;
  `pane.process_info` gives pid/argv/cwd only. `scripts/session-selftest.sh` — the
  verb-4 prover — asserts an exact `exit_code`. Through herdr that assertion is
  unsatisfiable except by injecting `; echo "MARKER exit=$?"` and scraping the
  marker back off the screen.
- **It cannot deliver signals.** No SIGINT/SIGTERM/SIGKILL. Only character-based
  `send_keys`, reaching the foreground process through the tty line discipline —
  it cannot target a pid. `termlink signal`, used by the prover's CLEANUP stage,
  has no equivalent.

Neither is an oversight to be patched. **Herdr models a human-watched pane;
TermLink models a programmatically-consumed command.** That is the whole
difference, and it runs the wrong way for us.

Today `exec` is a direct child and reads `status.code()`. Routing it through a
session server forces the `cmd_interact` marker-scraping shape
(`pty.rs:925–947`) — strictly weaker, and it breaks on binary output and on
commands that never return to a prompt. **Adopting herdr here would be a
regression on exactly the two capabilities verb 4 asserts.**

## 4. Correction to the earlier reframe

The recon claimed herdr overlaps three of four charter verbs. **That was
overstated on verb 1.** Herdr's agent state is heuristic screen-scraping — it
"reads the live bottom-buffer screen snapshot… evaluates TOML manifests against
that snapshot", and its own docs concede new prompts misclassify as `idle`
rather than `blocked`. TermLink's presence is a protocol with a liveness
contract and a canary watching it. Those are not the same kind of thing, and the
comparison flattered herdr. The real overlap is verb 4 only.

## 5. Maturity

- 141 days old, ~47 releases (~1 every 3 days)
- **Socket protocol went v8 → v14 within one minor step** (0.6.0 → 0.7.0);
  object-id semantics changed in 0.7.0, result types broke in 0.4.7
- **No stability guarantee** — no versioning or deprecation policy documented
- ~142 open issues, character = terminal-correctness bugs (ConPTY mouse loss,
  CSI-u leakage, OSC responses printed to screen, 80x24 pane floor), including
  **#2805 "pane.read always returns revision: 0"** — a live bug in the exact
  capture primitive we would bind to
- 29,247 stars against **91 watchers** — a trending spike, not deep adoption

A pre-stability dependency with a protocol that broke twice in four months is
the wrong foundation for the founding charter verb.

## 6. Self-inflicted risk worth naming

`session-selftest.sh:271` hardcodes `--backend tmux`. Remove tmux without
touching that line and **the prover breaks before the product does — the T-2557
canary goes dark, not red.** That is precisely the failure class this repo's
whole guard layer exists to catch, and here it would be self-inflicted. Any work
in this area must update the prover in the same change.

Also: `inject`'s delivered-vs-resolved distinction was hard-won across
T-2694/T-2697 with three load-bearing tests. Any replacement must reproduce the
*distinction*, not the happy path.

## 7. Coverage caveats — what was NOT established

Carried through rather than dropped, because a recommendation resting on
unstated gaps is the pattern this repo has been correcting all week.

- The internal agent was blocked by the budget gate on Bash **and** on dispatching
  helpers; it worked from `Read` alone on paths inferred from module manifests.
  Its positive findings are line-cited and solid; its **absences are bounded** —
  "zero in the files I opened", not "zero in the repo". **The exhaustive
  tmux call-site census and the fabric blast-radius were not delivered.**
- `pane.exited` payload is undocumented — the one place an exit code *might*
  surface. Needs an empirical test against a running server. If it does carry
  status, §3's first NO weakens (the signal NO stands regardless).
- Maintainer count inferred, not measured (contributors graph would not load).
- Relicense contributor consent: no CLA/DCO evidence either way.
- macOS quality genuinely ambiguous — README treats it tier-1 (`brew install`,
  not beta) but there is only ~1 substantive open macOS issue, which reads
  equally as "solid" or "few users". The external agent declined to pick, correctly.

**None of these gaps would flip the recommendation.** The two architectural NOs
in §3 and the pre-stability protocol in §5 are sufficient on their own, and the
unresolved items could at best soften one of the four reasons.

## 8. What to do instead

Nothing, on the evidence. tmux is a 12-line launcher that works, costs us
essentially nothing, and is proven daily by the T-2557 canary. There is no pain
here to relieve.

If the underlying interest is *"should TermLink still own verb 4 at all, given
tools like herdr now exist?"* — that is a legitimate and much more interesting
question, it connects to T-2468's over-built-in-breadth finding, and it belongs
in an inception task with a human go/no-go. It is not answered by this
evaluation, and it should not be decided by an agent.
