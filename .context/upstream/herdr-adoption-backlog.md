# Herdr adoption backlog — ranked (2026-08-15)

Synthesis of the five worker findings:
`herdr-adopt-{1-terminal-correctness,2-agent-state,3-parity,4-persistence,5-distribution}.md`.

**NO TASKS FILED.** This document is the ranking only. The orchestrator files tasks.

## Ordering rule, stated before the list

Ranked by **(probability the defect is live) × (cost of the wrong answer)** — NOT by
effort. Effort is recorded per item but never moves an item's position.

**Measured live defects outrank every speculative improvement, categorically.** Worker 1
demonstrated three of its findings against concrete inputs or a live syscall
(`openpty` NULL winp ⇒ rows=0/cols=0; a faithful port of `strip_ansi_codes` deleting
real characters). A defect that has been *observed producing a wrong answer* is not
commensurable with a defect that has been *reasoned about*, however elegant the
reasoning. Items 1–3 below are in the top three solely because they are measured. Item 4
is grep-proven absence (no emission site exists anywhere in the tree) — the strongest
non-measured evidence class, and it sits above every item whose evidence is inference.

Where a worker flagged its own claim `INFERRED` / unverified, that flag is carried into
the item text in **bold**, not smoothed away. Where two workers disagree, both readings
are stated. A recommendation resting on an unstated gap is the failure this repo has been
correcting.

---

# (a) IDEAS TO ADOPT — free, no licence obligation

All items below are category (a): a bug class, a test case, a design principle, or a
change to our own code. No herdr source is copied, transcribed, or structurally quoted.
Issue *titles* and doc *quotes* are cited as factual evidence that a class exists — that
is reference, not derivative use. **No attribution or NOTICE handling is triggered by
anything in this section.**

> **Status as of 2026-08-15.** Ranks **1–17 are CLOSED.** Closed this
> session: 9 (T-2737), 16 (T-2738), 10 (T-2740), 14 (T-2741), 15 (T-2742),
> 17 **and** 11 together (T-2743), 12 (T-2745, after T-2744 fixed its premise),
> 13 (T-2747) — plus **T-2739**, a third alt-screen surface
> (`mirror_grid.rs`) that this backlog did not list, found by grepping after 16
> and filed separately per one-bug-one-task.
> Remaining: **20–23**; **18 and 19 are `owner: human`**.
>
> **Rank 13's numbers were all wrong, including this document's.** The item cites
> "23 asserted pairs against 261 MCP tools ≈ 8.8%" and carries worker 3's
> unreconciled 94-vs-68 helper count. Measured: **260 tools, 24 asserted (9.2%)**,
> and **79** distinct `fn *_mcp` names — so none of 83 / 68 / 94 was right. The
> helper counts disagree because the unit is ill-defined: `fn to_json_mcp` alone
> occurs 26 times as a small helper redefined inside separate functions. **T-2747**
> therefore counts tools, not helpers, and enumerates all 236 uncovered ones in a
> git-tracked allowlist so a newly-added tool fires immediately. The
> name-mapping-heuristic caveat this document raises (`ping` diverging) **does not
> arise**: the census asks whether a tool is named by any assertion, never what its
> CLI counterpart is called. Working the 236 down is **T-2748**.
>
> **Rank 12's premise was wrong and it is now split.** Scoping it found that
> `metadata.termlink_version` — the field the item calls "data we already have" —
> recorded `termlink-session`'s Cargo.toml constant `0.9.0` on every build ever
> made, because that crate lacked the `build.rs` git derivation `termlink-cli`
> and `termlink-mcp` both carry. A detector over it would have compared a
> constant and answered "all stale" or "none stale" forever while looking
> healthy. **T-2744** fixed the field (measured: a probe session now records
> `0.11.1359`, matching `termlink --version` exactly); **T-2745** carries the
> detector with the corrected premise; **T-2746** covers the class — nothing
> detects the next crate that surfaces a version without the derivation.
> Read T-2745's Context before picking rank 12 up.
>
> **11 and 17 closed as one task, as this banner predicted.** 17 asks for a test
> of the survival property and 11 fixes the platform on which that property
> silently did not hold — a guard cannot be added for a property that is false,
> so they are one deliverable. The fix does not add the missing `nohup` the item
> describes; it removes the `setsid(1)` binary dependency entirely in favour of
> the `setsid(2)` syscall in `pre_exec`, which is POSIX and needs no fallback.
> That also moots worker 4's dispute with the platform-lock allowlist by
> deleting the three sites rather than re-arguing the recorded judgment — though
> worker 4 was right on the merits, and why is now PL-345.
> The UNREPRODUCED SIGHUP hazard was never load-bearing for the fix and remains
> untested as such; what is now tested is the property that makes SIGHUP
> irrelevant.

| # | Item | Evidence class | Effort | Directive |
|---|---|---|---|---|
| 1 | **`strip_ansi` mangles real text** (worker 1, class D) | **MEASURED** | ~1 d | D2 |
| 2 | **PTY spawns at 0×0** (worker 1, class A) | **MEASURED** | ~0.5 d | D2, D3 |
| 3 | **Preflight checks the wrong runtime_dir** (worker 5, F3) | READ, high-confidence | ~2 h | D2 |
| 4 | **No private-mode restore on detach** (worker 1, class B) | GREP-PROVEN ABSENCE | ~1–2 d | D2, D3 |
| 5 | **UTF-8 cut mid-char ⇒ U+FFFD** (worker 1, class I) | READ | ~0.5 d | D2 |
| 6 | **Inherited `TERMLINK_SESSION_ID` trusted unvalidated** (worker 1, class G) | READ | ~1 d | D2 |
| 7 | **Hub volatile-default warning is root-only** (worker 5, F2) | READ | ~1 h | D2 |
| 8 | **Child terminal queries unanswered — make the failure legible** (worker 1, class C) | GREP-PROVEN ABSENCE | ~0.5 d (test) | D2 |
| 9 | **Zombie survives `PtySession::drop`** (worker 1, class M) | READ | ~2 h | D1 |
| 10 | **Key-encoding table gaps** (worker 1, class E) | READ | ~1 d | D2, D3 |
| 11 | **`setsid` fallback drops `nohup` off Linux** (worker 4, R2) | READ + **UNREPRODUCED hazard** | ~0.5 d | D4, D2 |
| 12 | **Stale-binary session detector** (worker 4, R1 subset) | READ | ~0.5–1 d | D1, D2 |
| 13 | **Enumerating MCP/CLI parity census check** (worker 3, R1) | MEASURED coverage gap | ~1 d | D2 |
| 14 | **Remediation strings point at released binaries, not `cargo build`** (worker 5, F5a) | READ | ~1 h | D3, D4 |
| 15 | **Preflight Check 2 (`hubs.toml`) conditional** (worker 5, F4) | READ | ~2 h | D3 |
| 16 | **Alt-screen `?47` / `?1047` variants undetected** (worker 1, class J) | GREP-PROVEN ABSENCE | ~2 h | D2 |
| 17 | **"Survives SSH disconnect" has no test** (worker 4, §6.5) | ACKNOWLEDGED GAP | ~0.5 d | D1, D2 |
| 18 | **Default `runtime_dir` is the PL-021 footgun** (worker 5, F1) | READ — **human-owned** | ~1 d + migration | D1, D3 |
| 19 | **Shared response-envelope types in `termlink-protocol`** (worker 3, R2) | **human-owned; revisits T-2069** | weeks | D2, D3 |
| 20 | **`curl \| sh` user-level installer** (worker 5, F5b) | READ | ~1 d | D4, D3 |
| 21 | **`~/.termlink/hubs.d/<name>.toml`** (worker 5, F6) | READ — worker self-ranked last | ~1 d | D3, D4 |
| 22 | **VS-16 / wide-char mirror width** (worker 1, class F) | READ — display only | ~0.5 d | D3 |
| 23 | **`skip_state_update` as a remembered design idea** (worker 2, §6) | idea | 0 | D2 |

## Rationale per item

**1. `strip_ansi` mangles real text — MEASURED, rank 1.**
Two near-identical impls (`termlink-cli/src/util.rs:4–50`, `termlink-session/src/ansi.rs:5–46`)
terminate a CSI on `is_ascii_alphabetic()`, so a `~`-final sequence runs past its end and
`"\x1b[3~hello world"` → `"ello world"`. DCS/APC payloads are emitted **as text**.
Probability live = 1.0 (demonstrated on concrete inputs). Cost = maximal: this is a
surface whose entire contract is fidelity, and it fails in **both** directions — deletes
real characters AND injects escape-sequence payloads. Silent wrong answers on a
"give me the clean text" API. Ranked above the 0×0 winsize only because a 0×0 terminal
tends to produce *visibly* broken behaviour, whereas this produces *plausible* wrong text.
Add the test to **both** impls — the duplication is its own finding and a shared test is
what forces the merge.

**2. PTY spawns at 0×0 — MEASURED, rank 2.**
`pty.rs:106–114` passes `null_mut()` as `openpty`'s `winp`; grep found no product site
sizing it (`session.rs:279` spawns and never sizes). Measured: rows=0, cols=0. Everything
downstream of `termlink spawn --shell` runs in a zero-size terminal. Worker 1's honest
framing is worth preserving in the task: **herdr's #2828 is about an 80×24 floor; we have
no floor at all — their bug is our worse bug**, and their issue gave us the vocabulary to
look, rather than teaching us the defect.

**3. Preflight checks the wrong runtime_dir — rank 3, cheapest high-value item.**
`substrate-preflight.sh:240` hardcodes `${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}` and does
not replicate `discovery.rs:10–26`'s resolution order. On a normal systemd user session the
binary uses `/run/user/1000/termlink` while preflight reports on `/tmp/termlink-0` — an
unrelated path — and `/run/user/*` (a tmpfs destroyed at logout) matches neither `/tmp*`
nor `/var/tmp*`, so it would report **PASS: "persists across reboot"** if resolved.
This is the guard-reporting-green class (T-2680, T-2683) inside the check that exists to
prevent our single worst production failure mode. Ranked at 3 despite being ~2 h because
the cost of the wrong answer is "the operator believes PL-021 cannot happen here".
Fix: ask the binary (`termlink info --json` exposes `runtime_dir`) instead of re-deriving,
and add `/run/user/*` to the volatile set.

**4. No private-mode restore on detach — rank 4, strongest non-measured evidence.**
`pty.rs:487–489` / `:646–648` restore `termios` and nothing else. Grep for
`?1049|?100[0-6]|2004|kitty` finds **no emission site at all** in the tree — the only hits
are the alt-screen *detector* and a mirror-grid test. Detaching from a child that enabled
alt screen / mouse reporting / bracketed paste leaves the operator's terminal in that mode.
herdr **closed #2581** for this class, which is field proof it bites. Sits above items 5–6
because absence-by-grep is stronger evidence than a reading of present-but-suspect code.

**5. UTF-8 cut mid-char ⇒ U+FFFD — rank 5.**
`scrollback.rs:41–45` (`last_n_bytes`) and `:22–38` (ring-overflow `drain(..overflow)`) cut
at arbitrary byte offsets; `handler.rs:445` then `from_utf8_lossy`es. **Nuance that lowers
this below item 4:** `last_n_lines` cuts on `\n`, so the *default* path is safe — only
`--bytes` and ring overflow corrupt. But `cmd_interact` polls with `bytes: 131072`
(`commands/pty.rs:104–107`), so the corrupting path is on a live verb. The fix already
exists in-tree: apply `char_boundary_floor` (`commands/pty.rs:999`) at `scrollback.rs:41`.

**6. Inherited `TERMLINK_SESSION_ID` trusted unvalidated — rank 6.**
`metadata.rs:538–540`: `session_hint.or(env_hint).or(name_hint)` consumes the env var and
`find_session`es it with no check that the session owns the caller's process tree, and it
**short-circuits ahead of** the T-1303 PID-walk. Same shape at `tools.rs:11830`. The var is
seeded at `session.rs:277` and inherited by every descendant, so any process spawned under
a session inherits an identity claim. Cost is a wrong answer about *who you are* — but
exploitation requires an already-inside-the-session actor, which is why it sits below the
correctness items rather than above them. Fix both surfaces together per the T-2687
`parity_topics` lesson.

**7. Hub volatile-default warning is root-only — rank 7.**
`termlink-hub/src/server.rs:52–75` returns early unless `uid == 0`, justified by a comment
calling non-root `/tmp/termlink-UID` "not a footgun". `substrate-preflight.sh:242–282`
FAILs on `/tmp` **regardless of uid**. **Two guards disagree about whether the same state
is dangerous**, and PL-021's consequence (secret + cert regenerate on reboot) is identical
at any uid. ~1 h: remove one early return; the truth-table tests already inject uid.

**8. Child terminal queries unanswered — rank 8, scoped to legibility only.**
Nothing in the tree reads child output looking for DSR `CSI 6n` / `CSI 14t|16t` / OSC
10/11/4 and replies. A child that queries and waits blocks to the deadline with an empty
diff. **Scope this item to the test, not the responder:** assert `interact` returns a
timeout *naming the cause*. Building a responder is a larger design question (it means
TermLink starts pretending to be a terminal emulator) and should be a separate decision.
Worker 1's cluster analysis is the right framing: A/B/C share one root — **TermLink models
a PTY nobody is watching**, which is charter-correct, and is exactly why nothing sizes it,
tears down its modes, or answers it.

**9. Zombie survives drop — rank 9.** `pty.rs:456–467` `SIGKILL`s then *immediately*
`waitpid(WNOHANG)`; the child has almost certainly not been reaped in that instant, so the
zombie survives to process exit. Bounded cost (fd/PID pressure in long-lived hosts), high
probability. Needs a blocking wait or short retry.

**10. Key-encoding table gaps — rank 10, deliberately below the correctness items.**
`executor.rs:242–292` is missing Ctrl+I/J/M, Ctrl+[/]/^/_, all F-keys, Shift+Tab (CSI Z),
PageUp/PageDown, Insert, modified arrows. **It is ranked here and not higher because it
fails LOUD** — `resolve_key` → `None` → `"Unknown key: {name}"` (`executor.rs:297`). That
is D2-correct behaviour: a refusal, not a silent drop. The task should **pin the loud
failure first** (that assertion is load-bearing and currently untested) and only then
widen the table.

**11. `setsid` fallback drops `nohup` — rank 11, carries an UNREPRODUCED hazard.**
`execution.rs:541–556` runs `setsid sh -c` and on spawn error falls back to a **bare
`sh -c` with no `setsid` and no `nohup`**. macOS ships no `setsid` (util-linux), which
`.context/checks/platform-lock-allowlist:26` states. On a headless macOS host with no tmux,
`Auto` → `Background`, the fallback fires, and the session is left in the invoking shell's
session. Our own `scripts/be-reachable.sh:258–261` wraps **both** arms in `nohup`; the Rust
path does not — same decision, two answers, weaker one on the charter-verb path.
**Worker 4 explicitly flags that it did NOT test whether SIGHUP reaches this child** —
whether it dies depends on the parent shell's job-control behaviour. Treat this as
*hardening* until a 5-minute macOS test settles it; describing it as a bug fix before that
would be the unstated-gap failure. Worker 4 also **disputes an existing accepted
allowlist entry** — arguing lines 30–31's "degraded but functional" is incomplete against
T-2693's own rule, because *functional for spawning* is not *functional for surviving
disconnect*. That is a live disagreement with a decision this repo already recorded, and
should be resolved explicitly rather than by silently editing the allowlist.
Fix retires **three** platform-lock entries via `libc::setsid()` in `pre_exec`.

**12. Stale-binary session detector — rank 12.**
The proportionate subset of herdr's `update --handoff` insight (§1c): a session daemon
needs a story for its binary being replaced while it holds live processes. Full fd-passing
handoff is **weeks and not proposed**. But `Registration.metadata.termlink_version` is
already written (`registration.rs:220`, `:286`), so `termlink sessions --stale-binary` is a
read over data we already have — T-2359's version-floor pattern at the session tier.
Verified absence: every `handoff|re-exec|live.migrat` hit in the tree is `claim-transfer`,
an unrelated concept.

**13. Enumerating parity census check — rank 13.**
23 asserted pairs against 261 MCP tools ≈ **8.8%**; the other ~238 are not passing, they
are **unexamined**, and nothing in the output distinguishes those two states — the T-2680
lesson one layer up. The guard layer (12 canaries, 6 static checks) has **no member that
knows the two surfaces are supposed to correspond** (`ls scripts/ | grep -i parit` → NONE;
7 allowlists, no parity one). Adopt the seven-times-proven shape: enumerate what exists,
fire on anything neither covered nor allowlisted with a cited reason. It does not prevent
drift; it converts *unexamined* into *acknowledged*.
**Two carried caveats.** (i) Worker 3 counts **94** `*_mcp` helpers where `parity.rs`'s own
header says **68**; the difference is unreconciled and neither is asserted correct — both
are grep-shaped and may count different things. (ii) Name-mapping is heuristic; `ping`
already diverges (CLI positional `[TARGET]` vs MCP `target` param, while CLI `--target`
means something else). Expect a first run that is mostly allowlist entries.
Worker 3 is also precise about a distinction worth keeping: **`parity_topics` WAS covered
and DID catch T-2624 — what failed from 2026-08-12 is that nothing ran it.** T-2686 closed
the *execution* gap; this closes the *coverage* gap. Different bugs.

**14. Remediation strings — rank 14, highest visibility per hour in the list.**
`substrate-preflight.sh:407,415,428` tell the operator to `cargo build --release`. We
already publish macOS + Linux binaries and a Homebrew formula; the operator-facing strings
just do not use them. Independent of the installer (item 20) and most of its benefit.

**15. Preflight Check 2 conditional — rank 15.**
`config.rs:112–128` treats missing `hubs.toml` as a normal empty config and the local hub
is reached at `runtime_dir()/hub.sock` with no profile. A purely-local install never needs
the file, yet Check 2 WARNs whenever it is absent — telling a correctly-configured operator
something is wrong on first run. Alert-fatigue prevention, the PL-219 class this repo
already names.

**16. `?47` / `?1047` — rank 16.** `pty.rs:399–400` matches only `?1049h/l`. Grep for the
older variants: zero. Extend the existing split-read tests at `pty.rs:621–669`, already the
right shape. Narrow class (older/simpler TUIs), cheap.

**17. "Survives SSH disconnect" has no test — rank 17.**
Worker 4 flags this as an open gap it did not close, and notes it **became more interesting
in light of item 11**: the survival property is argued from the mechanism (`setsid` /
`tmux -d` / Terminal.app), which is sound but is not a test — and item 11 identifies a
platform where that mechanism silently degrades. **This is the guard that would have caught
item 11.** Worth doing independent of whether 11 or 12 proceeds.

**18. Default `runtime_dir` — rank 18, HUMAN-OWNED, the real prize.**
`discovery.rs:10–26` resolves `$TERMLINK_RUNTIME_DIR` → `$XDG_RUNTIME_DIR/termlink` →
`$TMPDIR/termlink-$UID` → `/tmp/termlink-$UID`. **Three of four candidates are volatile**
(`$XDG_RUNTIME_DIR` = `/run/user/$UID`, a tmpfs destroyed at logout). The largest production
failure class in this repo is what happens **when the operator configures nothing**.
Herdr's `~/.config/herdr/` is persistent by construction, so the class cannot occur there
regardless of configuration — the one genuinely instructive contrast in worker 5's slice.
Defaulting to a persistent per-user path *removes* the class rather than detecting it (D1).
It ranks 18 not because it is unimportant — it is the highest-value item in worker 5's
slice — but because it **changes a default live deployments rely on** and needs a migration
path for hubs already holding a live `hub.secret` under the old default. Inception-shaped,
human go/no-go before code. Items 3 and 7 are the detect-layer fixes that make this safe to
sequence behind; **item 18 is also the prerequisite for auto-start (see (c)).**

**19. Shared envelope types — rank 19, HUMAN-OWNED, revisits a standing decision.**
The duplication is **not in the operation** — `termlink-mcp/Cargo.toml:10–27` has no
`termlink-cli` dependency and both crates already sit over `termlink-protocol`/`-session`/
`-hub`. It is in the **response envelope**: `events.rs:1210–1213` and `tools.rs:14033–14038`
are two hand-written `json!` literals with the same key names. That is the whole T-2624 bug
and it recurs exactly as often as someone edits one literal. Sharing the serialised type
makes forgetting one *impossible* rather than *untested*.
**Carried explicitly: this proposes revisiting the T-2069 convention** ("no cross-crate
sharing for these tiny pure helpers"), which produced the 94 duplicate helpers. **That
convention is the drift generator**, and changing it is a human decision, not an agent one.
Also carried: worker 3's mechanism claim about herdr — that its CLI is a client of its own
server, so one implementation exists — is **explicitly labelled INFERRED at moderate
confidence**. Worker 3 found **no codegen, no shared schema, no macro dispatch table**, did
not read `src/api/` or `src/cli/`, and notes both doc pages assert the shared surface as a
property without explaining a mechanism. The recommendation does not depend on the
inference being right (the T-2624 evidence is ours and local), but the framing "herdr solved
this" does, and must not be stated as fact in a task.
Do item 13 first — not because it is the fix, but because **nobody currently knows how big
the surface is that this would have to cover**, and 8.8% reported as green is how we got
here.

**20. `curl | sh` installer — rank 20.** Independently implemented, herdr's shape only
(single binary, `~/.local/bin`, no sudo, os/arch from `uname`, writes no config, does not
edit shell rc). Optional; item 14 delivers most of the benefit for an hour.

**21. `hubs.d/` — rank 21, last by the worker's own assessment.** T-2681 already validated
bundled-with-local-override in this repo (`.context/checks/<name>-allowlist`, tracked, env/
flag override wins). Applying it to `hubs.toml` is ergonomics for an operator who already
got through onboarding — it does not serve the onboarding question that motivated the slice.

**22. VS-16 / wide-char width — rank 22, and the honest move may be to decline.**
`mirror_grid.rs:187` drops combining/zero-width marks ("*silently drop for v1*") and
`unicode-width = "0.1"` predates emoji-presentation width, so `➡️` measures 1 and draws 2.
**Display-only — the byte stream is untouched.** Worker 1's own recommendation is to pin
current behaviour with a test and then *decide*, rather than take a `unicode-width` major
bump for a mirror. Do not let this become a dependency upgrade by momentum.

**23. `skip_state_update` — rank 23, zero cost, nothing to build.**
A rule that matches, is reported, and explicitly **declines to change state**. Same instinct
as our exit-2 tooling class (T-2557: *"a check that never looked must never read as a clean
bill"*). Worth remembering if an inference surface is ever built. No code, no licence
exposure, no task needed — record it as a learning.

---

# (b) CODE TO COPY — Apache-2.0 attribution + NOTICE handling required

## **EMPTY. Nothing in this backlog requires copying herdr code.**

This is not an oversight and it is not hedging — **all five workers independently and
explicitly declared their entire slice category (a)**, and each stated the licence position
in its own header. Every item in section (a) is either a test against our own code, a change
to our own defaults, or a design principle. Issue titles and documentation quotes are cited
as evidence that a class exists; that is factual reference, not derivative use.

**No `NOTICE` file change, no attribution header, and no third-party-licence review is
triggered by adopting any item ranked above.**

Two places where category (b) *would* arise, recorded so a future task does not cross the
line without noticing:

1. **Vendoring herdr's detection engine** — `src/detect/manifest.rs` (~1,500 LOC) or the 20
   bundled `*.toml` manifests. Worker 2 quotes the manifest *format* as evidence and
   explicitly flags that adopting it becomes category (b) (Apache-2.0 at v0.8.0). This is
   rejected on the merits anyway — see (c) R1.
2. **Reading herdr's handoff implementation** — worker 4 notes the docs say *what*
   `update --handoff` does, never *how* (fd passing? re-exec? `SCM_RIGHTS`?). Item 12 is
   scoped to a read-only detector precisely so this question does not arise. **If item 12
   is ever widened to an actual live handoff, the licence question opens at that moment**
   and must be re-flagged before any herdr source is read for implementation guidance.

If any task derived from section (a) later proposes lifting herdr's *implementation* rather
than reproducing the *idea*, it moves to this section and needs attribution + NOTICE
handling before a line is written.

---

# (c) TEMPTING BUT REJECTED

### R1 — Heuristic agent-state detection wired into `agent-presence` / `find-idle` / `/peers`
**Rejected. This is the strongest and most consequential rejection in the set.**

Three independent disqualifying reasons, any one sufficient:

1. **It fails OPEN, toward the most expensive answer.** `manifest.rs:497–534`: when a
   *recognised* agent matches no rule, the reported state is `Idle`. A stale manifest, a
   redesigned prompt, a resized pane pushing chrome out of `bottom_non_empty_lines(5)`, an
   agent between spinner frames — all report "available, send it work". `find-idle` is the
   input to `/claim` → `/claim-transfer`, so work gets assigned to an agent blocked on a
   permission prompt, and the lease then expires into the stuck-claims canary — a second
   guard absorbing damage from the first.
2. **It permanently latches an existing canary.** A non-opted-in agent emits no heartbeat,
   so making it visible requires **synthesising an `agent-presence` heartbeat the agent
   never sent**. Such an entry has no `pty_session`, so it fires T-2387 class (a)
   LIVE-but-unwakeable (`check-waker-liveness-freshness.sh:121–146`) **permanently, with no
   operator action able to clear it** — the remediation is precisely the opt-in the agent
   declined. **This repo just fixed this exact bug**: T-2709 narrowed the stuck-claims
   canary off `expired_count > 0` because it was a monotonic latch, *"the precise mechanism
   by which a guard teaches its operator to stop reading it."* Rejecting this is
   **consistency with a decision already made, not caution.**
3. **The two "idle"s are different quantities, not two implementations of one predicate.**
   TermLink `idle` = "heartbeating AND holding no claim" — a bookkeeping fact from two
   protocol records (`lib.rs:558–655`, `:643–647`). herdr `idle` = "the bottom of the screen
   looked like a prompt, or nothing matched." A second weaker producer also re-forks the
   liveness predicate T-2585 deliberately unified across all three discovery surfaces.

Cost is weeks (~1,500 LOC engine + a VT grid for region extraction + 20 manifests tracked
against upstream churn), and manifests are **fetched remotely** — classification can change
from a network fetch with no local code change (a D4 concern worker 2 flagged but did not
cost). **Fails D1 and D2.**

**Stated plainly because it is true and should not be lost in the rejection:** herdr can
tell you an agent is **blocked waiting on a human**, and TermLink cannot — not even for
opted-in agents. Our model has no blocked/working axis at all. That is a **genuine
capability gap**. It is simply not a *presence* gap, and answering it through the presence
rail is what makes the proposal unsafe.

### R2 — A separate, non-authoritative observational verb for blocked-detection
**Rejected as unasked — and the rejecting worker wrote the proposal itself.**
Worker 2 §7 designs the only safe shape (separate verb over sessions we already own PTYs
for, own confidence field, never written to `agent-presence`, never read by `find-idle`,
never counted by a canary), then declines to recommend it: it serves no requested need, and
T-2468 found TermLink **over-built in breadth** with T-2548 still open on ~28 off-charter
tools. **Adding an inference surface nobody asked for while a human decision to subtract
surface is pending is exactly the accretion the charter-drift canary (T-2483) exists to
catch.** If ever wanted, it enters as an inception with a human go/no-go — not as a
fallback smuggled into a rail three canaries depend on.

### R3 — herdr's single-binary shape (MCP server as a client of the CLI, or vice versa)
**Rejected.** Would mean rewriting 45,799 LOC of `tools.rs` as a subprocess or socket shim,
and would import herdr's own constraint — **the CLI can only express what the socket can
express** — into a codebase whose `exec` path deliberately keeps a direct-child
`status.code()`. That constraint is not hypothetical: it is why herdr has **neither exit
codes nor signals**, the two architectural NOs already settled in the synthesis. D4 also cuts
against it — the MCP surface is the portable one and should not be downstream of a CLI's
argument grammar. **Item 19 takes the invariant (share the serialised type) without the
architecture.**

### R4 — `herdr --remote` SSH-bridged thin client
**Rejected.** Solves "render a remote TUI locally". TermLink's remote story is RPC to the hub
over TCP and its clients are **agents, not humans at a TUI**. Breadth accretion with no
charter verb behind it — the T-2468/P4 finding, the T-2483 class. Worth noting what
`--remote` actually is, since it is easy to overrate: the "private per-attach control
socket" is an **SSH ControlMaster** socket, not a forwarded API socket. The herdr socket API
never crosses the network; what crosses is a rendered UI stream. It is `ssh host herdr
attach` with keepalives — ergonomics, not architecture.
**One transferable scrap, not a recommendation:** herdr adds SSH keepalives by default
because it learned the channel is the fragile part. If any TermLink workflow depends on a
long-lived `ssh` — the `ssh:` `bootstrap_from` reauth anchor is the candidate — the same
hardening may apply. **Worker 4 did not investigate this**; it is flagged, not claimed, and
appears in OPEN QUESTIONS rather than in section (a).

### R5 — Zero-auth local socket
**Rejected.** Herdr's socket has no auth, token, or pinning of any kind. Our secret + TOFU
pinning is the cross-host trust model. Worker 5's framing is the one to keep: **herdr's
install is easy partly because it defends nothing**, and most of TermLink's install cost buys
something herdr does not offer. Only the subset where we pay cost and get *nothing* back is
adoptable — that subset is items 3, 7, 14, 15, 18.

### R6 — Auto-spawning a hub on first client call
**Rejected FOR NOW — sequenced, not refused on the merits.** Verified: nothing auto-starts a
hub (`auto_start|ensure_hub|spawn_hub` → 0; `cli_integration.rs:1583,2196,4187` assert
commands *fail* when no hub runs). It is the closest thing to herdr's `herdr`-just-works
loop and is genuinely attractive. But **a hub silently self-starting under the wrong
`runtime_dir` is precisely how PL-021 propagates.** Item 18 is a hard prerequisite. Revisit
only after the default is persistent.

### R7 — herdr's nested `all`/`any`/`not` manifest gate grammar
**Rejected: nothing to apply it to.** Worker 2's own verdict — an elegant grammar with no
predicate in our codebase that needs it. Recorded so it is visibly declined rather than
silently omitted.

### R8 — Adopting herdr's `SHELL`-unset behaviour as a fix
**Rejected: there is nothing to fix.** herdr filed `SHELL` unset ⇒ `/bin/sh` as a bug
(#2641); `pty.rs:88–90` does the identical thing. Listed because it *looks* like a free
finding in the issue corpus and is not one — a herdr issue is evidence a class exists, not
evidence our behaviour is wrong.

### R9 — Full fd-passing live session handoff across binary upgrade
**Rejected at this scope.** Weeks of work, and it re-opens the data-plane and registration
invariants. Item 12 (stale-binary *detector*, ~0.5–1 d) is the proportionate subset and is
what should be built. Recorded here because "herdr has `update --handoff` and we don't" is
a true and tempting one-liner that hides an enormous scope difference.

---

# OPEN QUESTIONS — not actionable yet

Things a worker could not verify. **None of these is converted into a recommendation
above.** Each names who could not verify it and what would settle it.

1. **herdr issue BODIES were never read — only titles** (workers 1 and 4). Every mapping
   from a herdr issue to a TermLink class is inference from a title. Worker 1 names #2828
   ("80x24 floor") and #2581 ("leaks CSI sequences into parent shell") as the two it would
   most want to read in full **before quoting them in a task**, because both are
   load-bearing for a HIGH row. Worker 4 says the same of its §2 table: #2775 and #2006 are
   strong on the title alone, the rest are weaker. *Settled by: reading ~6 issue bodies.*
2. **"Fails today" is code-derived, not red-test-derived, for four of worker 1's items.**
   Only item 2 (0×0 winsize) is *measured* against a live syscall and only item 1
   (`strip_ansi`) is *demonstrated* against concrete inputs. Item 4's evidence is
   grep-proven absence (strongest of the rest). Items 5, 6, 10 and the class-G/E claims are
   read from code — **worker 1 explicitly states it did not run the test suite.**
   *Settled by: writing the tests, which is the first task anyway.*
3. **Worker 3's central mechanism claim is INFERRED.** "herdr's CLI is a client of its own
   server, so exactly one implementation of each operation exists" — no codegen, no shared
   schema, no macro dispatch table was found; `src/api/` and `src/cli/` were not read; both
   doc pages assert the shared surface without explaining a mechanism. *Settled by: reading
   `src/cli/` — though note item 19 does not depend on it.*
4. **94 vs 68 `*_mcp` helpers, unreconciled** (worker 3 vs `parity.rs:14-15`'s own header).
   Both counts are grep-shaped and may count different things; neither is asserted wrong.
   *Settled by: item 13's census, which has to enumerate them anyway.*
5. **Does SIGHUP actually kill the `setsid`-less fallback child?** (worker 4, R2). Reasoned,
   **not reproduced.** The kernel HUPs the session leader and the controlling terminal's
   foreground process group; this child is stdio-null'd and backgrounded, so whether it dies
   depends on the parent shell's job-control behaviour. *Settled by: a 5-minute test on a
   macOS host without tmux — and that test should precede describing item 11 as a bug fix
   rather than a hardening.*
6. **Is the platform-lock allowlist reason at lines 30–31 adequate?** Worker 4 argues
   "degraded but functional" fails T-2693's own rule because *functional for spawning* is
   not *functional for surviving disconnect*. **This is a disagreement with an
   already-recorded repo decision**, not a new finding. *Settled by: a human ruling on the
   allowlist entry, ideally together with Q5.*
7. **Class K — does the diff renderer re-emit stale cells?** (worker 1). `pty.rs:779` calls
   `grid.render_diff` per frame with no periodic full-repaint escape hatch found, but
   **`mirror_grid.rs` was not read end-to-end**. A structural risk note, explicitly *not* a
   confirmed defect. herdr has #2793/#2795/#1914 in this class. *Settled by: reading
   `mirror_grid.rs`.*
8. **macOS was never separately assessed for classes A/B/M** (worker 1). Given T-2692's
   macOS CI is still **non-blocking** and T-2693 exists precisely because `/proc`
   assumptions leaked in, these deserve a macOS-specific read that no worker performed.
   Overlaps Q5. *Settled by: a macOS pass, or by promoting T-2692's CI job.*
9. **Windows/ConPTY classes were excluded as out-of-scope, NOT as handled** (worker 1). A
   large share of herdr's traffic (#2810, #2536, #2726, #1183, …) is ConPTY. We are
   `libc::openpty` + `fork` and do not target Windows — which makes those issues
   *irrelevant*, not *passed*. Recorded so a later reader does not mistake the silence for
   coverage.
10. **Does herdr auto-spawn its server on first client call?** (worker 5). A
    `auto_detect_launch()` was mentioned in one fetch; the socket-api page, fetched
    directly, **does not document auto-spawn** and explicitly left it unaddressed.
    *Suggested by two sources, confirmed by neither.* Bears on R6 only.
11. **herdr's handoff implementation is undescribed** (worker 4). Docs say what
    `update --handoff` does, never how. *Settled only by reading their source — and that is
    where the category-(b) licence question opens.* Do not read it casually.
12. **Is the remote-manifest fetch path signature-verified?** (worker 2). `manifest_update.rs`
    was not audited. Flagged as a D4 consideration, not costed. Bears on R1 only, which is
    rejected regardless.
13. **Do our long-lived `ssh:` `bootstrap_from` anchors need keepalive hardening?**
    (worker 4, R3). Flagged, **not investigated**. The only transferable scrap from
    `--remote`. *Settled by: a look at the reauth anchor path.*
14. **"Survives SSH disconnect" is argued from mechanism, not proven by any test** (worker 4,
    §6.5). Listed as item 17 in section (a) as a *test to write* — but until it is written,
    the property remains unproven, and Q5 identifies a platform where the mechanism silently
    degrades.
15. **Worker 4 did not read `pty.rs` itself** — its `pty.rs:103–120` and
    `commands/pty.rs` citations are carried second-hand from worker 1's full read. Line-cited
    and almost certainly fine, but it is a single-source chain, not two independent reads.
16. **Neither worker 2 nor worker 5 ran herdr.** All behaviour is read from source at pinned
    `v0.8.0` (worker 2) or from unpinned live doc pages through WebFetch's summarising model
    (workers 3, 4, 5). **Direct quotes are reliable; a page's silence is not proof of
    absence** — worker 4 states this explicitly and notes it never established herdr's
    socket path despite reading two pages that would naturally have mentioned it.
