# T-2267 — Cross-Hub Communication: Deep Review (Strategy + Implementation)

**Date:** 2026-06-24
**Trigger:** Recurring, repeated-across-cycles failures for agents to communicate
cross-hub (ring20 .122 and peers). Operators/agents keep hitting the wall, getting
a misleading error, and misdiagnosing the cause as "I have no TermLink identity."
**Method:** Live probing of .122 + three parallel read-only code-audit agents
(scope/permission model, transport/auth/resolution, messaging UX).

---

## Verdict

**The cross-hub transport is sound. The instability is entirely in the three layers
above the wire — authorization, resolution, and diagnostics — and every single
failure mode misdirects the caller.** That is why capable agents repeatedly conclude
"I can't reach the hub / I have no identity" and give up: the substrate tells them
the wrong thing at every step. There is also a **central missing abstraction**: no
single sanctioned "message a remote agent by name" verb exists.

A caller trying to reach a remote agent today walks into this gauntlet:
1. `agent contact <name>` → resolves **local sessions only** → "not found."
2. Falls back to raw `host:port` → secret is **not reverse-resolved** → "secret required."
3. Switches to the profile name, calls a *read* method → **denied, requires `execute`**
   (a list shouldn't need execute) → reads like a param/identity rejection.
4. Cert rotated? The TOFU violation is **buried** behind "is the hub running?"
5. Old hub build? `-32601 Method not found` is **indistinguishable** from a typo.

Five steps, five misleading signals. The product is unstable because it lies about
why it failed.

---

## Layer 1 — Authorization (scope model)  ·  IMMEDIATE FIX

| # | Sev | Finding | Location |
|---|-----|---------|----------|
| 1 | HIGH | All **16 `channel.*` methods** fall through to the deny-by-default `_ => Execute` arm — read-only `list/state/info/subscribe/receipts/claims/cv_keys` wrongly require `execute`. | `auth.rs:202`, `server.rs:372-389` |
| 2 | HIGH | **`agent.find_idle`** (substrate DISPATCH verb #2) — pure read — also requires `execute` via the same fall-through. | `control.rs:282` |
| 3 | MED | Also unmapped → wrongly `execute`: `artifact.get/put`, `event.emit_to`, `dialog.presence`, `orchestrator.*`. | `control.rs:73,96-100,291,301,308` |
| 4 | HIGH | **Scope is self-asserted** — `create_token` accepts any caller-chosen scope with no ceiling; any secret-holder mints `execute` freely. So misclassifying reads as `execute` provides **zero** security benefit and **only** harms honest downscoped callers. Production code already hardcodes an `execute` token *just to perform reads* (`channel.rs:323`) — proof the misclassification is a known pain. | `auth.rs:272`, `channel.rs:323` |
| 5 | LOW | **Two drifting scope tables** (`auth.rs::method_scope` + `server.rs::hub_method_scope`) are the structural enabler — the entire `channel.*` family slipped through the gap between them. | both files |

**Correct classification (the fix payload):**
- **Observe:** `channel.list, subscribe, receipts, claims, claims_summary, cv_keys`, `agent.find_idle`, `artifact.get`
- **Interact:** `channel.post, claim, renew, release`, `event.emit_to`, `artifact.put`
- **Control:** `channel.create, set_retention, transfer_claim, force_release, trim, sweep`

## Layer 2 — Resolution (secret / profile / address)

| # | Sev | Finding | Location |
|---|-----|---------|----------|
| 6 | HIGH | **Bare `host:port` never reverse-resolves the secret** in MCP/CLI-remote paths (they bail the instant they see `:`), yet `channel post --hub` *does* match the address field and auto-loads it. Same hub: works by profile-name, fails by address. Inconsistent across three resolution paths. | `tools.rs:6687-6690`, `config.rs:81-89` vs `channel.rs:247-274` |
| 7 | MED | IP-keyed secret cache (`~/.termlink/secrets/<ip>.hex`, G-011) is never consulted at call time and has no drift hint — a stale cache silently auth-fails after a hub restart. | `tools.rs:6779-6799,10122` |

## Layer 3 — Diagnostics (error taxonomy)

| # | Sev | Finding | Location |
|---|-----|---------|----------|
| 8 | HIGH | **TOFU cert-drift is swallowed into "is the hub running?"** — the real "TOFU VIOLATION … run `tofu clear`" message is buried in `{e}` behind a misleading connectivity framing. | `client.rs:91`, `tools.rs:6849-6852` |
| 9 | HIGH | **`-32601 Method not found` is indistinguishable from old-hub-version** — a richer `-32011 PROTOCOL_VERSION_TOO_OLD {declared,required,method}` exists but `remote_call` never special-cases it or probes remote version. This is the `remote_exec` "method doesn't exist" confusion verbatim. | `tools.rs:14587-14590`, `control.rs:366-369` |
| 10 | MED | `-32010` cannot distinguish bad-secret vs stale-secret vs insufficient-scope; the caller can't tell auth from authz. | `tools.rs:6866-6868`, `control.rs:365` |

## Layer 4 — Abstraction (the missing verb)

| # | Sev | Finding | Location |
|---|-----|---------|----------|
| 11 | HIGH | **No single cross-hub "message agent by name" verb.** `agent contact <name>` resolves names only via the local filesystem registry; cross-hub forces the operator to hand-supply `--target-fp` + `--hub`. The pieces to auto-wire exist (`agent_listeners_fleet` returns `(hub, fp)` per row) but nothing chains them. | `agent.rs:817`, `tools.rs:17461` |
| 12 | HIGH | **MCP `termlink_agent_contact` cannot reach cross-hub at all** — hardcodes the local socket, exposes no `hub` param, despite advertising `target_fp` "for cross-host peers." | `tools.rs:17491,17516` |
| 13 | HIGH | **`agent-send.sh --to` auto-discovery is single-hub** — calls `agent-listeners.sh` (local) not `-fleet`, so a peer on another hub yields "no listener" — looks like "agent offline" when it's "agent on another hub." | `agent-send.sh:96-98` |
| 14 | MED | dm: topic discovery is guesswork for remote peers — topic = `dm:<sorted_fp_a>:<sorted_fp_b>`; no verb maps "remote agent name → its dm topic." | `channel.rs:715-722` |
| 15 | MED | Authz is **host-coarse, not agent-scoped** — the hub secret bypasses per-agent authz; a host with the secret posts as its host key for any session. | `channel.rs:323,503-504` |
| 16 | LOW | Not-found/offline error messages steer toward broadcast (`--mention` on chat-arc), never "peer may be on another hub — try fleet presence." | `agent.rs:828-849` |

---

## Prioritized fix sequence

1. **Scope-map fix + regression test + actionable `-32010`** — *this task, T-2267.*
   Classify the full `channel.*` surface + `agent.find_idle` + the other unmapped
   reads; add a scope-matrix test so any future method hitting the `Execute`
   catch-all fails CI; enrich `-32010` with remediation. **Unblocks every
   read-scoped cross-hub caller immediately.** Low risk, high value.
2. **Diagnostics honesty** (findings 8, 9, 10) — surface TOFU drift verbatim;
   on `-32601` probe remote version and annotate skew vs typo; split `-32010`
   bad-credential vs insufficient-scope. *Follow-up task.*
3. **Bare-address reverse-resolution** (finding 6) — make MCP/CLI-remote match the
   address against hubs.toml like `channel post --hub` already does. *Follow-up.*
4. **The missing verb** (findings 11-14, 16) — cross-hub `agent contact <name>`:
   resolve name → `(hub, fp)` via fleet presence → mint token → dm post; add `hub`
   to MCP `termlink_agent_contact`; switch `agent-send.sh --to` to the fleet
   variant. *Sizeable build — its own task(s).*
5. **Self-asserted scope ceiling** (finding 4, 15) — bind a max scope to the
   secret/issuer so the tiers become a real control, not advisory ergonomics.
   **Design decision — human/inception, not an autonomous fix.**

Items 1-3 are pure reliability/usability wins with no model change. Item 4 is the
strategic UX fix. Item 5 is a security-model decision for the human.
