# Independent review

## 1. Diagnosis

Mostly right, with one real gap. Confirmed: unsigned `channel subscribe` on `inbox:*` records `(unnamed, local, claimed)` exactly as the registry shows; cron at 06:47Z matches `last 2026-11-21T06:47:08Z`; two inboxes read 23 ms apart is script-like, not human; `instance local` matches a root cron job (a TCP reader would record `tcp:<ip>` — so the reader is on the hub host).

Contradiction / gap: `first 2026-11-20T05:52:37Z` is not HH:47. The registry's merge key is `agent+instance+strength+key_fp` (record(), inbox_consumers.rs), so **every local unsigned process merges into one row** — the canary, a 05:52 manual read, or a second recurring local reader (log shipper, another cron) are indistinguishable. The bundle does not rule out a recurring ~05:52 reader; the agent's original "one-off manual reads" story is actually compatible with 05:52:37 seeding the entry and the canary refreshing it.

One settling check: observe the registry's unnamed `last_ms` for 48h and correlate every advance against the canary log mtime (HH:47:08 local). Any advance at another timestamp is a second reader. Zero changes, no guard touched. (Note `record()` writes admin rows only for evictions/binds, so there is no richer audit log to grep.)

## 2. Options A–D

**A — skip all `inbox:`.** Weakness (high): an agent mails files as `document`/`artifact-chunk` messages to any inbox (direct mail is the most natural FX-5 bypass) and the canary is blind there permanently; nothing replaces that coverage. Rule: narrowing counts as switching off — violates the rule unless paired with a proven replacement (see F) plus documented approval.

**B — skip inboxes with a proven single reader.** Weakness (high, and fatal as worded): the consumer list *today* shows proven **plus** the canary's own unnamed row inside the 24h window, so the predicate "shows a proven single reader" is false every day → B never skips → the window never drains → **37 never passes; B deadlocks on its own residue**. It only works if the predicate is `bound` or "the owner's proven entry exists." Secondary (medium): check-then-read TOCTOU and the silent false-clean below. Rule: same narrowing as A, smaller class, still no replacement for the skipped coverage; also adds unproven conditional logic to a guard — the opposite of "proven in the field."

**C — teach 37 to tolerate the canary.** The agent's blindness claim is overstated: the registry *does* distinguish `local` from `tcp:<ip>`. Weakness (high): tolerating unnamed-local readers tolerates any compromised local process on the trust-anchor host, and the registry/sidecar status stay permanently dirty, so every other consumer of P-12 still sees a violation. Rule: retires a working check's teeth with no replacement — violates the spirit.

**D — do nothing.** Weakness (high): 37 and sidecar status are permanently red; because the unnamed row always exists, a real intruder hides in the canary's noise — D destroys P-12's signal exactly like C, with no code change, plus red-normalization on every deploy. Rule: violates nothing on paper; in substance leaves a live P-12 violation running.

## 3. Missed options

- **E — canary signs its reads (own key card, `read_proof`); P-12/37 admit a named, revocable audit reader.** Costs: key custody for a cron job, code in the canary (termlink-session plumbing exists) and in 37's allowlist, and a documented rule amendment (one-reader → one-reader + named audit). Answers C's objection: an intruder can't appear as the canary without its key, and a second *unnamed* reader still fails 37. Coverage stays 100%, so it best respects the standing rule. **37 passes: yes, by the amended rule.**
- **F — the W1 sidecar runs file-shape detection on its own inbox** (same predicate, surfaced in `sidecar status`). Costs: logic shipped in each sidecar, upgrade coupling, per-inbox scope only. This is the *proven replacement* the standing rule demands before any narrowing. **37 passes: no — must be paired with A or B after field proof (fixture parity + N days of agreeing counts).**
- **G — read only topic metadata for inboxes (no subscribe).** `channel list` gives name+count only; count can't detect file shapes. It is A with extra steps (a bare count-growth signal at best). Costs: same coverage hole as A. **37 passes: yes**, after the window drains.

## 4. Option B: races and failure modes

- **Bootstrap deadlock** (high): as worded, the canary's own in-window row defeats the predicate. Test: fixture = today's registry (owner proven + unnamed, `last_ms` 23h59m ago) → canary must skip.
- **Bind appears between consumers-call and subscribe** (high): FCW-124 refusal; the script's parser treats a JSON-RPC error as zero rows → topic looks empty → **false clean**, violating "never report clean on tooling error" (`tooling()` only catches subprocess exceptions). Fix: detect error objects, exit 2. Test: subscribe fixture returning an error.
- **Bind disappears between call and read** (medium): canary reads, reseeds the unnamed row, 37 red for 24h. Fail-loud, self-heals; pin with a stubbed race fixture.
- **consumers call errors/timeouts** (FCW-125 unreadable, table full, timeout) (high if skip-on-error — false clean; the only rule-compliant choice is **exit 2**). Test both fixtures.
- **Inbox on another hub** (low): local `consumers()` returns an empty list for unknown topics (`unwrap_or_default`); local subscribe of a nonexistent topic records nothing. Pin with a fixture: listed-but-unreadable topic ⇒ exit 2 or 0, never a crash.
- **Exit code when skipping** (medium if unspecified): skip is not firing and not tooling ⇒ 0 with heartbeat; a run that skips everything is still 0. Test mixed fixtures.
- **State-file interaction (affects A too)** (medium-high): `--ack` overwrites state with only this run's findings, so a skipped topic **loses its acked offset**; when the skip later lifts (sidecar down), all historical file-shaped posts count as new → alert flood. Fix state merge before any skip ships.

## 5. Verdict

I'd choose **F then A, staged**: build sidecar-side file-shape detection for its own inbox, prove it in the field (fixtures plus N days of counts agreeing with the canary), get documented operator approval, then land the unconditional inbox skip — the window drains in ≤48h, 37 goes green honestly, direct-mail detection is *replaced* rather than lost, and there is no B-shaped TOCTOU machinery to maintain. If 37 must pass before F can be proven, the interim is E (sign the canary, amend P-12's rule once), which keeps coverage and keeps 37 able to catch unnamed readers. Steelman of B: it is the smallest change that turns 37 green without weakening any check's semantics or amending a rule — the skip tracks proven ownership automatically, so newly bound inboxes drop out by construction and stale unbound ones stay covered. With the predicate fixed to `bound`, error ⇒ exit 2, and RPC-error detection added, its race windows are milliseconds wide and fail loud — a defensible quick fix if the operator accepts standing coverage loss on the two production inboxes pending F.
