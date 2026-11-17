# T-3704 design review round 1 — dispositions

Seats: opencode (`review-1-opencode.md`), claude-code (`review-1-claude-code.md`), isolated, tools off, bundle 1.
No item was judged WRONG by either seat. Every WEAK item and every missed finding is answered below: A = accepted and
changed in the design note (v2), R = rejected with the reason, D = deferred to slice B with the reason, Q = an operator
question (asked one at a time). Numbers: O-n / C-n are the seats' items; M-n their missed findings.

| # | Finding | Disp. | Change in design v2 |
|---|---|---|---|
| O-1, C-1 | liveness type gated but no agent extraction defined | A | `sidecar.liveness.v1` dropped from the gate: AL-33 scopes the check to "an arc-011 message type to inbox:*", and liveness posts go to `liveness:*`. Gate = `sidecar.msg.v1` and `sidecar.stage.v1` on `inbox:*`. |
| C-1 | prefixed card key compared with raw `sender_pubkey_hex` | A | compare the decoded 32 bytes, never strings. |
| C-M1 | `from_agent` used as a file path (traversal) | A | `from_agent` must match `^[a-z0-9][a-z0-9._-]{0,63}$` and contain no `..`, else refused like a missing card (FCW-135), at hub and receiver; cards live in ONE store file keyed by name, never a path built from the name. |
| C-3 | `card.project` never compared; overwrite can change project | A | a put that changes an existing card's `project` (or its key) is refused unless it says `replace` explicitly; every replace raises attention and an admin row naming old and new; fixture with two projects (Q's agent claiming P's agent name refused). |
| O-4, O-M1, C-4 | gate keyed on poster-chosen msg_type; other types fail open (BP-3) | A (+R part) | The arc-011 receiver admits only `sidecar.msg.v1` / `sidecar.stage.v1` (it already ignores other types); a new mutant proves that admitting an unlisted type is caught. So no arc-011 mail can be forged under another type. AEF's rail on `inbox:*` (other types) keeps today's behaviour (never-decommission rule); carding AEF traffic is AEF's design and goes to them as a request (R for this unit, filed). |
| O-M4, C-5 | default `off`; no owner of `enforce`; setting changes not logged | A | setting changes are admin-logged and shown in `hub.status`; P-31 (T-3713) gains the criteria "move to `log` after cards publish" and "move to `enforce` after 7 days of log with zero false refusal"; the R-75 floor is met at P-31's close, not at slice A's. |
| O-9, C-9 | `execute` scope is not a runme credential; agents run as root | Q | No barrier is possible while agents run as root (same uid, same files). Compensating controls added: every card put/replace raises a needs-attention entry to the operator and an admin row; the hub refuses a put over the network (unix socket only). Operator question: accept this as a stated limit until non-root agents, or hold slice A. |
| O-11, C-11 | receiver: remote senders not in the local register; card fetch needs AL-20 | A | the receiver binds `from_agent` to the key through a NAMED roster (`roster: {agent: key}`); a name in the roster with another key is refused (FCW-98); an unnamed legacy roster entry is accepted as today and counted "unbound" in status (receiver follows the same off/log/enforce setting). No card fetch in slice A. |
| C-12 | forger gets the refusal; impersonated agent learns nothing; log flood | A | the hub counts refusals per claimed agent in `hub.status`; the claimed agent's sidecar reads that on its status tick and raises a needs-attention entry "posts claiming to be you were refused" (coalesced per day); admin rows coalesced per (claimed agent, poster key fingerprint, minute). |
| O-M3, C-M5 | AL-33 2.17.f crash/read-lock invariants dropped; durability; corrupt card | A | store under `<runtime_dir>` (= /var/lib/termlink on every hub, durable, T-1294); start-up compares each card's `seq` with the last logged `seq` and raises attention on a mismatch; a post reads the card under the per-agent lock; an unreadable or corrupt store refuses every gated post (fail closed) with the reason, never treats it as "no cards" (E-169 fake case). |
| O-M5 | runme put route untested; admin-row-unwritable case | A | tests: put accepted and logged; put refused when the admin row cannot be written (FCW-128 shape). |
| C-15 | `ed25519:<64 hex>` looks like the hub fingerprint form | A | card keys are `ed25519-pub:<64 hex>` / `x25519-pub:<64 hex>`; the fingerprint keeps `ed25519:<sha256>`. |
| O-7, C-7, O-M2, C-M2, C-slicing | slice B and E-70 undesigned; P-16's done criterion needs all E-n | D | Slice B is designed in its own note before its code, after P-15 lands AL-20. T-3704 stays open until slice B passes: P-16 is done only when E-69 (both halves), E-70 and E-169 pass. |
| C-8 | per-project DEAD seq and multiple instances | D | slice B design; recorded. |
| C-6 | remote home hub's DEAD | D | slice B; DEAD from another hub needs that hub's pinned key (PH-3 out-of-band pinning, 18.2.15). |
| C-M3 | revocation not checked at the hub on post | D | revocation is P-15; slice B adds the hub-side revocation check once P-15's list exists (architecture 6.2.5). |
| C-M4 | old-key replacement moves the card but not the register (P7-2); 6.6.5 says that route is PH-3 while E-69 wants it in PH-2 | Q (later) | a spec conflict; asked when slice B is designed, after the slice A question. |
| C-13 | no AL-20 vectors for key-replace / DEAD yet | D | slice B adds vectors when it adds the schemas. |
| C-10 | DEAD in the sender vocabulary | D | slice B adds `DEAD` at `api.rs:44-57`. |
