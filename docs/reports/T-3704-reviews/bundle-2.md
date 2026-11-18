# T-3704 slice A build review bundle (round 2: the code)

Unit: an agent key card at the hub and a post-time check that an arc-011 post is signed by the key on the card of the agent it claims to be (FCW-133/135), plus the receiver-side name binding (FCW-98). Below: measured results, the operator ruling on card writes, the design note (v2), the round-1 design-review dispositions, the mutant table, the code diff, and the two real-hub test files. Slice B (old-key replacement, signed dead letter) is out of scope.

## Measured results (2026-11-18)
- hub lib tests 620+ passed, 0 failed; key_card unit tests 13/13; router t3704 test 1/1
- sidecar full suite 333 passed, 0 failed; wire t3704 tests 2/2
- real-hub tests: t3704_key_card 2/2; t3704_impersonation 1/1
- mutants: 16 of 16 killed

## Operator ruling on card writes (recorded in the task's Decisions, 2026-11-18)

- **Operator words:** "A" (on the brief recommending A; design review r1 items O-9 / C-9).
- **Chose:** A — build slice A now. Card writes are unix-socket only, admin-log row first (refused if unwritable), a needs-attention entry to the operator on every put/replace naming old and new key, and a change of an existing card's key or project only with explicit `replace`. **Stated limit:** while agents run as root, a root agent can write a card the way runme does; the controls make that visible, not impossible. The limit closes when agents run non-root.
- **Rejected:** B (hold slice A until non-root agents); C (operator approval with a digest for every card write).

# T-3704 (arc-011 P-16, B-6) — design note: agent key card, post-time check, signed dead letter

Source of the spec: `docs/reports/T-3704-spec-extract.md` (verbatim B-6, R-44/R-62/R-75, E-69/E-70/E-169, AL-33,
AL-34, AL-23 step 4, AL-20, and a map of the current code). This note settles the 15 gaps that extract lists, splits the
unit into two slices, and states what each slice proves. Written 2026-11-17, before any code.

## 1. What exists today (measured)

1. The hub already verifies every `channel.post` signature, but against the key the poster supplies
   (`sender_pubkey_hex`), and requires `sender_id == fingerprint(key)` (`channel.rs:906-968`). Any fresh key passes:
   nothing binds a key to an agent or a project. That binding is what B-6 adds.
2. The arc-011 sidecar signs posts with the agent's own key over `canonical_sign_bytes` (`wire.rs:213`), the same bytes
   the hub verifies; the claimed agent (`from_agent`) and instance are only inside the signed payload (`wire.rs:39-53`).
3. Agent public keys are already provisioned by runme as root-owned register files
   (`/etc/termlink-sidecar/keys/<agent>.pub`, P7-2, T-3438), and the encryption keys likewise (`<agent>.x25519.pub`).
4. No AL-20 (RFC 8785) implementation exists in `crates/`; P-15 (T-3703, captured) is the first unit that needs it.
5. The hub has no instance registry and no notion of "retired".

## 2. Slices

The unit splits cleanly at AL-20:

- **Slice A (this task, now): the card and the post-time check.** A card store at the hub, written by a runme-only
  admin path; every post of an arc-011 message type to `inbox:*` is checked against the card of the agent it claims to
  be from (FCW-133, FCW-135); the receiver checks `from_agent` against the key too (AL-23 step 4). Needs no AL-20:
  the card is written by a privileged path, not by a signed statement. Delivers R-75 "verified as P's agent" and the
  E-69 post half and E-169's "poster without a card".
- **Slice B (after P-15 has built AL-20): key replacement by old-key statement (FCW-134) and the signed DEAD letter
  (AL-34, FCW-136, E-70).** Both are AL-20-signed statements; building AL-20 twice would fork the canonical form that
  P-15's test vectors pin. So the order becomes P-16 slice A, then P-15 (T-3703), then P-16 slice B. The plan lists
  P-16's only dependency as P-10 (built), so slice A is in order.

## 3. The 15 gaps, settled (numbers as in the extract)

1. **Card key.** Cards are keyed by agent name; one card per agent name per hub, enforced at registration. The hub
   reads `from_agent` from the `sidecar.msg.v1` / `sidecar.stage.v1` payload (it already parses stage payloads in
   `mail_guard.rs`) and requires `card.signing_key == sender_pubkey_hex`; the existing signature verify then stands
   as is. A post whose payload does not parse is refused like a post without a card (fail closed).
2. **Bytes verified.** The transport signature stays (`canonical_sign_bytes`); the card adds key == card key. AL-20 is
   used only for signed hub state (slice B).
3. **Project binding (R-75.e).** Agent names are unique per hub (gap 1), and the card carries `project`; a session of
   project Q that claims to be P's agent must post with P's agent key, which it does not hold. The receiver also checks
   `from_agent` against the key (gap 11). No wire change.
4. **Which message types.** Exactly `sidecar.msg.v1`, `sidecar.stage.v1` and `sidecar.liveness.v1` (the last on
   `liveness:*`). Every other type on `inbox:*` (AEF's `sidecar.receipt`, chat posts, hub-originated envelopes) is
   untouched: the check never sees them (never-decommission rule).
5. **Roll-out.** A hub setting with three values: `off` (default in this build), `log` (record what would be refused,
   refuse nothing), `enforce`. P-31 publishes the cards; the operator moves the setting to `log`, then `enforce`, by
   runme, after the log shows no false refusal. "Check off at the hub; cards kept" is the rollback.
6. **DEAD needs the hub public key.** Slice B: `hub.version` gains `signing_public_key` (64 hex); the sidecar checks
   sha256 of it against its pin before using it. (The pin stays the fingerprint.)
7. **Where DEAD lives.** Slice B, settled there with P-15's form; recorded as open.
8. **DEAD seq rule.** Slice B: refuse a lower seq; an equal seq only for byte-identical content (R-60.a's wording;
   AL-34's `>` read as "rises or is the same record").
9. **Who may write a card.** An admin RPC `hub.key_card_put` that accepts only `execute` scope over the local unix
   socket, writes an admin-log row (B-3) before it answers, and is called only by runme. While agents run as root
   this is a convention plus a log, not a barrier: a root agent could call it. Recorded as a known limit (RR-2 class),
   visible in the admin log; the barrier comes with the non-root agent work.
10. **Name clash.** The new state is `DEAD` (R-44); the existing sidecar poison event stays `DEAD_LETTER`. Slice B
    labels them apart in status.
11. **Receiver.** The receiver keeps its roster (the keys allowed to send) and adds the missing check: the signer key
    must be the key of the `from_agent` the message claims (the receiver reads the same register files P7-2 provisions,
    or the hub's card over the pinned path). This closes "post as `fx-a` signed with `fx-b`'s key" at the receiver
    when both keys are rostered.
12. **Needs-attention at the hub.** The hub writes an admin-log row per refusal and counts refusals in `hub.status`;
    the sidecar raises the needs-attention entry when its post is refused (it already does for refusals).
13. **Canonical JSON.** Slice A signs nothing new. Slice B uses P-15's AL-20 implementation with the V-REV vectors.
14. **Two levels.** E-169 (L2) on the hub fake; E-69 (L4) on a throwaway real hub, as T-3366 did.
15. **Key format.** The card stores `signing_key` and `encryption_key` as `ed25519:<64 hex public key>` and
    `x25519:<64 hex>`.

## 4. Slice A, what gets built (v2, after review round 1: `T-3704-reviews/r1-dispositions.md`)

Where section 3 and the dispositions differ, the dispositions win (v2): liveness is out of the gate; card keys are
`ed25519-pub:` / `x25519-pub:`; the receiver uses a named roster; P-16 closes only when slice B passes too.

1. Hub: ONE card store file `<runtime_dir>/key-cards.json` (0600, atomic write + fsync, never a path built from a
   name), schema `termlink.key-card/1` per entry: `agent, project, signing_key, encryption_key, instances, seq, by`.
   Per-agent lock; posts read the card under it; `seq` compare-and-set on write. Start-up compares each `seq` with the
   last logged `seq` (attention on mismatch). An unreadable or corrupt store refuses every gated post with the reason.
2. Hub: `hub.key_card_put` / `hub.key_card_get` / `hub.key_card_list`. Put: unix socket only, `execute` scope, admin
   row written before the answer (refused if it cannot be written), needs-attention raised on every put; a put that
   changes an existing card's project or key needs `replace: true` and names old and new.
3. Hub: post check in `handle_channel_post` for `sidecar.msg.v1` / `sidecar.stage.v1` on `inbox:*`, behind
   `off|log|enforce` (default `off`; changes admin-logged; shown in `hub.status`). `from_agent` validated as a plain
   name; refusals `CHANNEL_NO_KEY_CARD` (FCW-135) and `CHANNEL_KEY_CARD_MISMATCH` (FCW-133), admin rows coalesced per
   (claimed agent, poster key, minute); per-claimed-agent refusal counts in `hub.status`.
4. CLI: `termlink hub key-card put|get|list` and `termlink hub key-card-check off|log|enforce` for runme.
5. Sidecar: named roster `{agent: key}`; `from_agent` must be a plain name and, when named in the roster, signed by
   that key (FCW-98); unnamed legacy entries accepted and counted "unbound"; admits only the two arc-011 types; reads
   its own refusal count from `hub.status` and raises "posts claiming to be you were refused" (coalesced per day).
6. Tests: E-169 card cases on the fake (no card; corrupt store; admin row unwritable on put); E-69 post half on a
   throwaway real hub (`fx-a` signed with `fx-b`'s key; hub secret only; no card; Q's agent claiming P's name; a
   traversal name) at the hub and, with the hub check off, at the receiver; put accepted and logged; replace without
   `replace: true` refused. Mutants: `fc-01-verify-accepts-any-signature`, one per refusal, the name check, the
   receiver type filter, the lock-free read, the unreadable-store-as-empty.
7. No runme action in this slice: publishing cards and moving the setting are P-31 (T-3713).

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

## Mutants
| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| FCW-133 / E-69.5 | `t3704-m1-fc-01-verify-accepts-any-signature` | the card key is not compared with the signer (plan control fc-01) | `key_card::tests::e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes` | killed |
| FCW-128 | `t3704-m10-put-not-admin` | card writes are not admin RPCs (no intent row first) | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |
| FCW-98 / E-69.4 | `t3704-m11-receiver-ignores-bound-name` | the receiver does not check a bound from_agent against the key | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| FCW-98 | `t3704-m12-open-path-skips-names` | an unrostered key may claim a bound agent name | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| design v2 C-12 | `t3704-m13-no-refusal-counts` | refusals are not counted for the impersonated agent | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| AL-33 step 2 (admin_log) | `t3704-m14-refusals-never-logged` | no admin row is ever written for a refusal | `key_card::tests::refusal_rows_are_coalesced_per_agent_key_minute` | killed |
| design v2 C-12 (sidecar) | `t3704-m15-impersonation-silent` | the impersonated agent's sidecar never raises the attention entry | `a_refused_post_claiming_to_be_me_raises_one_attention_entry` | killed |
| FCW-135 / E-169 | `t3704-m2-no-card-passes` | a poster without a card passes | `key_card::tests::e169_fcw135_a_poster_without_a_card_is_refused` | killed |
| design v2 C-M1 | `t3704-m3-no-name-check` | from_agent is used without the plain-name check | `key_card::tests::a_traversal_name_is_refused_and_never_read_as_a_path` | killed |
| design v2 O-M3 | `t3704-m4-corrupt-store-as-empty` | a corrupt card store reads as no cards | `key_card::tests::an_unreadable_store_refuses_never_reads_as_empty` | killed |
| design v2 C-3 | `t3704-m5-replace-not-required` | a card key or project changes without replace | `key_card::tests::a_key_or_project_change_needs_replace_and_seq_rises` | killed |
| design v2 (fail closed) | `t3704-m6-bad-setting-is-off` | an unreadable or invalid setting reads as off | `key_card::tests::mode_defaults_off_and_a_bad_setting_enforces` | killed |
| FCW-133 at the hub | `t3704-m7-enforce-stores` | in enforce mode a refused post is stored anyway | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| NID | `t3704-m8-gate-every-type` | the check also gates other message types and topics | `key_card::tests::other_types_and_topics_are_never_gated` | killed |
| operator ruling A (local socket only) | `t3704-m9-put-over-tcp` | a card write is accepted from a TCP peer | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |
| AL-33 2.17.f(2) | `t3704-m16-no-startup-reconcile` | a card ahead of the admin log is never reported at start | `key_card::tests::al33_startup_reconcile_flags_a_card_ahead_of_the_admin_log` | killed |

PASS: all 15 mutants turned their test red. Run 2026-11-18T00:56Z on f213ffd7a.
Mutant 16 (start-up reconcile) added after the run above: killed, 2026-11-18T01:06Z. Total: 16 of 16 killed.

## Code diff (crates/*/src)
```diff
diff --git a/crates/termlink-cli/src/cli.rs b/crates/termlink-cli/src/cli.rs
index 5406aecf4..f5503dfe0 100644
--- a/crates/termlink-cli/src/cli.rs
+++ b/crates/termlink-cli/src/cli.rs
@@ -1445,6 +1445,54 @@ pub(crate) enum TmuxAction {
     },
 }
 
+/// `termlink hub key-card` actions (T-3704).
+#[derive(Subcommand)]
+pub(crate) enum KeyCardAction {
+    /// Write (or, with --replace, change the key or project of) one agent's card
+    Put {
+        /// Agent name (plain: [a-z0-9][a-z0-9._-]{0,63})
+        #[arg(long)]
+        agent: String,
+        /// The agent's project id
+        #[arg(long)]
+        project: String,
+        /// ed25519-pub:<64 hex> — or --signing-key-file with the raw 64-hex public key (P7-2 register)
+        #[arg(long, conflicts_with = "signing_key_file")]
+        signing_key: Option<String>,
+        /// File holding the agent's raw 64-hex ed25519 public key (e.g. /etc/termlink-sidecar/keys/<agent>.pub)
+        #[arg(long)]
+        signing_key_file: Option<std::path::PathBuf>,
+        /// x25519-pub:<64 hex> (optional)
+        #[arg(long)]
+        encryption_key: Option<String>,
+        /// Instance ids of the agent (repeatable)
+        #[arg(long = "instance")]
+        instances: Vec<String>,
+        /// Allow changing an existing card's key or project (raises attention, names old and new)
+        #[arg(long)]
+        replace: bool,
+        #[arg(long)]
+        json: bool,
+    },
+    /// Show one agent's card
+    Get {
+        agent: String,
+        #[arg(long)]
+        json: bool,
+    },
+    /// List all cards, the check setting, and refusal counts per claimed agent
+    List {
+        #[arg(long)]
+        json: bool,
+    },
+    /// Show the check setting, or set it: off | log | enforce
+    Mode {
+        mode: Option<String>,
+        #[arg(long)]
+        json: bool,
+    },
+}
+
 /// Hub server actions
 #[derive(Subcommand)]
 pub(crate) enum HubAction {
@@ -1627,6 +1675,13 @@ pub(crate) enum HubAction {
         #[arg(long)]
         json: bool,
     },
+    /// Agent key cards at the local hub (T-3704, arc-011 B-6): write, read, list, and the
+    /// off|log|enforce check setting. Writes go over the local unix socket only, are admin-logged,
+    /// and raise attention; runme is the intended writer.
+    KeyCard {
+        #[command(subcommand)]
+        action: KeyCardAction,
+    },
     /// Probe a remote hub via TLS handshake and print its leaf cert sha256
     ///
     /// G-011 rotation-protocol companion to `hub fingerprint` (which reads
diff --git a/crates/termlink-cli/src/commands/infrastructure.rs b/crates/termlink-cli/src/commands/infrastructure.rs
index 3a9617393..4ffbae918 100644
--- a/crates/termlink-cli/src/commands/infrastructure.rs
+++ b/crates/termlink-cli/src/commands/infrastructure.rs
@@ -985,6 +985,61 @@ pub(crate) fn cmd_hub_fingerprint(json_output: bool) -> Result<()> {
 
 /// T-3366: `termlink hub adopt-id` — the AL-17 legacy migration, the only path past FCW-79.
 /// Exit 0 with what it did (or would do); an unreadable/malformed id or key file is an error.
+/// T-3704: `termlink hub key-card put|get|list|mode` against the local hub socket. A refusal
+/// exits non-zero with the hub's reason (runme reads it).
+pub(crate) async fn cmd_hub_key_card(action: crate::cli::KeyCardAction) -> Result<()> {
+    use crate::cli::KeyCardAction as A;
+    use termlink_session::client;
+    let socket_path = termlink_hub::server::hub_socket_path();
+    let (method, params, json_out) = match action {
+        A::Put { agent, project, signing_key, signing_key_file, encryption_key, instances, replace, json } => {
+            let key = match (signing_key, signing_key_file) {
+                (Some(k), _) => k,
+                (None, Some(f)) => {
+                    let raw = std::fs::read_to_string(&f).with_context(|| format!("reading {}", f.display()))?;
+                    let hex = raw.trim().to_ascii_lowercase();
+                    if hex.len() != 64 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
+                        anyhow::bail!("{} does not hold a raw 64-hex public key", f.display());
+                    }
+                    format!("{}{hex}", termlink_hub::key_card::SIGNING_PREFIX)
+                }
+                (None, None) => anyhow::bail!("give --signing-key or --signing-key-file"),
+            };
+            (
+                "hub.key_card_put",
+                json!({"agent": agent, "project": project, "signing_key": key,
+                       "encryption_key": encryption_key, "instances": instances, "replace": replace}),
+                json,
+            )
+        }
+        A::Get { agent, json } => ("hub.key_card_get", json!({"agent": agent}), json),
+        A::List { json } => ("hub.key_card_list", json!({}), json),
+        A::Mode { mode, json } => match mode {
+            Some(m) => ("hub.key_card_mode", json!({"mode": m}), json),
+            None => ("hub.key_card_list", json!({}), json),
+        },
+    };
+    let resp = tokio::time::timeout(std::time::Duration::from_secs(10), client::rpc_call(&socket_path, method, params))
+        .await
+        .map_err(|_| anyhow::anyhow!("{method}: the local hub did not answer within 10s"))??;
+    let result = client::unwrap_result(resp).map_err(|e| anyhow::anyhow!("{method}: {e}"))?;
+    if json_out {
+        println!("{result}");
+    } else if let Some(cards) = result.get("cards").and_then(|c| c.as_array()) {
+        println!("mode: {}", result["mode"].as_str().unwrap_or("?"));
+        for c in cards {
+            println!("{}  project={}  key={}  seq={}", c["agent"].as_str().unwrap_or("?"), c["project"].as_str().unwrap_or("?"),
+                c["signing_key"].as_str().unwrap_or("?"), c["seq"]);
+        }
+        if let Some(r) = result.get("refusals").and_then(|r| r.as_object()).filter(|r| !r.is_empty()) {
+            println!("refusals: {}", serde_json::Value::Object(r.clone()));
+        }
+    } else {
+        println!("{}", serde_json::to_string_pretty(&result).unwrap_or_default());
+    }
+    Ok(())
+}
+
 pub(crate) fn cmd_hub_adopt_id(dry_run: bool, expect: Option<&str>, json_output: bool) -> Result<()> {
     let dir = termlink_session::discovery::runtime_dir();
     let a = termlink_hub::hub_identity::adopt_legacy(&dir, dry_run, expect)
diff --git a/crates/termlink-cli/src/main.rs b/crates/termlink-cli/src/main.rs
index 23e5c0ff9..570d2e15a 100644
--- a/crates/termlink-cli/src/main.rs
+++ b/crates/termlink-cli/src/main.rs
@@ -311,6 +311,7 @@ async fn real_main() -> Result<()> {
             Some(HubAction::DispatchResolve { ref dispatch_id, attempt, ref as_state, ref reason, ref override_reason, ref outcome, ref evidence, json }) => commands::infrastructure::cmd_hub_dispatch_resolve(dispatch_id, attempt, as_state, reason, override_reason.as_deref(), outcome.as_deref(), evidence.as_deref(), json),
             Some(HubAction::Fingerprint { json }) => commands::infrastructure::cmd_hub_fingerprint(json),
             Some(HubAction::AdoptId { dry_run, expect, json }) => commands::infrastructure::cmd_hub_adopt_id(dry_run, expect.as_deref(), json),
+            Some(HubAction::KeyCard { action }) => commands::infrastructure::cmd_hub_key_card(action).await,
             Some(HubAction::Probe { addr, json }) => commands::infrastructure::cmd_hub_probe(&addr, json).await,
         },
         Command::Main { action } => {
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index eed006ff9..66fe80a01 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -969,6 +969,20 @@ pub(crate) async fn handle_channel_post_with_peer(
         .into();
     }
 
+    // T-3704 (arc-011 B-6, 06a AL-33): an arc-011 post must be signed by the key on the card of
+    // the agent it claims to come from (FCW-133 / FCW-135), behind the off|log|enforce setting.
+    if let Some(resp) = key_card_gate(
+        &termlink_session::discovery::runtime_dir(),
+        &id,
+        &topic,
+        &msg_type,
+        &payload,
+        &verifying_key,
+        &expected_fp,
+    ) {
+        return resp;
+    }
+
     // T-1287: optional metadata routing-hint map. NOT included in canonical
     // signed bytes — trusted-mesh threat model treats it as routing only.
     // Well-known keys: conversation_id, event_type (per T-1288 catalog).
@@ -2873,6 +2887,58 @@ fn hex_lower(bytes: &[u8]) -> String {
     bytes.iter().map(|b| format!("{b:02x}")).collect()
 }
 
+/// T-3704: the key-card gate for one post whose transport signature verified. `None` = go on.
+/// In `log` mode a would-be refusal is counted and logged but the post is stored; in `enforce`
+/// (or with an unreadable setting) it is refused. Admin rows are coalesced per (agent, key, minute).
+pub(crate) fn key_card_gate(
+    dir: &std::path::Path,
+    id: &Value,
+    topic: &str,
+    msg_type: &str,
+    payload: &[u8],
+    signer: &VerifyingKey,
+    signer_fp: &str,
+) -> Option<RpcResponse> {
+    use crate::key_card::{self, Check, Mode};
+    let mode = key_card::effective_mode(dir);
+    if mode == Mode::Off {
+        return None;
+    }
+    let Check::Refuse { code, fcw, agent, reason } = key_card::check_post(dir, topic, msg_type, payload, signer.as_bytes())
+    else {
+        return None;
+    };
+    let enforced = mode == Mode::Enforce;
+    key_card::note_refusal(&agent, enforced);
+    let now = std::time::SystemTime::now()
+        .duration_since(std::time::UNIX_EPOCH)
+        .map(|d| d.as_millis() as u64)
+        .unwrap_or(0);
+    if key_card::first_in_minute(&agent, signer_fp, now) {
+        let row = json!({
+            "event": if enforced { "key-card-refused" } else { "key-card-would-refuse" },
+            "fcw": fcw, "agent": agent, "signer": signer_fp, "topic": topic,
+            "msg_type": msg_type, "reason": reason, "mode": mode.as_str(),
+        });
+        if let Err(e) = crate::rpc_audit::admin_note(row) {
+            tracing::error!(error = %e, "T-3704: key-card refusal admin row not written");
+        }
+    }
+    tracing::warn!(%agent, fcw, %reason, mode = mode.as_str(), "T-3704: arc-011 post failed the key-card check");
+    if !enforced {
+        return None;
+    }
+    Some(
+        ErrorResponse::with_data(
+            id.clone(),
+            code,
+            &format!("refused: {reason} ({fcw})"),
+            json!({"fcw": fcw, "agent": agent}),
+        )
+        .into(),
+    )
+}
+
 fn parse_pubkey_hex(hex: &str) -> Option<VerifyingKey> {
     if hex.len() != 64 {
         return None;
diff --git a/crates/termlink-hub/src/key_card.rs b/crates/termlink-hub/src/key_card.rs
new file mode 100644
index 000000000..e4bd115b9
--- /dev/null
+++ b/crates/termlink-hub/src/key_card.rs
@@ -0,0 +1,546 @@
+//! T-3704 (arc-011 P-16, B-6, 06a AL-33): agent key cards at the hub and the post-time check.
+//!
+//! A card binds an agent name to its signing key and project. Cards live in ONE store file
+//! (`<runtime_dir>/key-cards.json`, 0600, atomic replace), never in a path built from a name
+//! (design v2, review C-M1). They are written only by `hub.key_card_put`, an admin RPC accepted
+//! over the local unix socket (operator ruling A 2026-11-18: while agents run as root this is a
+//! visible, logged convention, not a barrier).
+//!
+//! The check runs on every `channel.post` of an arc-011 type (`sidecar.msg.v1`,
+//! `sidecar.stage.v1`) to `inbox:*`; every other type and topic is never seen by it (NID). It is
+//! behind a hub setting `<runtime_dir>/key-card-check` = `off` (default, file absent) | `log` |
+//! `enforce`. The poster's agent is the `from_agent` of the signed payload; it must be a plain
+//! name, have a card, and the card's key must be the key that signed the post (the transport
+//! signature is verified before this check, unchanged).
+
+use std::collections::{BTreeMap, HashSet};
+use std::path::{Path, PathBuf};
+use std::sync::Mutex;
+
+use serde::{Deserialize, Serialize};
+use termlink_protocol::control::error_code;
+
+pub const STORE_FILE: &str = "key-cards.json";
+pub const MODE_FILE: &str = "key-card-check";
+pub const CARD_SCHEMA: &str = "termlink.key-card/1";
+pub const STORE_SCHEMA: &str = "termlink.key-cards/1";
+/// The arc-011 message types the check sees (design v2: liveness posts go to `liveness:*`).
+pub const GATED_TYPES: [&str; 2] = ["sidecar.msg.v1", "sidecar.stage.v1"];
+pub const SIGNING_PREFIX: &str = "ed25519-pub:";
+pub const ENCRYPTION_PREFIX: &str = "x25519-pub:";
+
+/// One lock for the store: a post reads its card under it, and a write is a read-modify-write
+/// under it, so a post meets either the old or the new card, never a half-written one (AL-33
+/// 2.17.f(3)).
+static STORE_LOCK: Mutex<()> = Mutex::new(());
+
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Mode {
+    Off,
+    Log,
+    Enforce,
+}
+
+impl Mode {
+    pub fn as_str(self) -> &'static str {
+        match self {
+            Mode::Off => "off",
+            Mode::Log => "log",
+            Mode::Enforce => "enforce",
+        }
+    }
+    pub fn parse(s: &str) -> Option<Mode> {
+        match s.trim() {
+            "off" => Some(Mode::Off),
+            "log" => Some(Mode::Log),
+            "enforce" => Some(Mode::Enforce),
+            _ => None,
+        }
+    }
+}
+
+/// The setting. Absent file = `off`. A file that cannot be read or holds anything else is an
+/// `Err`: the caller treats that as `enforce` (fail closed), never as `off`.
+pub fn read_mode(dir: &Path) -> Result<Mode, String> {
+    match std::fs::read_to_string(dir.join(MODE_FILE)) {
+        Ok(s) => Mode::parse(&s).ok_or_else(|| format!("{MODE_FILE} holds {:?}, not off|log|enforce", s.trim())),
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Mode::Off),
+        Err(e) => Err(format!("{MODE_FILE} unreadable: {e}")),
+    }
+}
+
+/// The effective mode for a check: an unreadable or invalid setting enforces.
+pub fn effective_mode(dir: &Path) -> Mode {
+    read_mode(dir).unwrap_or(Mode::Enforce)
+}
+
+pub fn write_mode(dir: &Path, mode: Mode) -> std::io::Result<()> {
+    atomic_write(&dir.join(MODE_FILE), format!("{}\n", mode.as_str()).as_bytes(), 0o644)
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct Card {
+    pub schema: String,
+    pub agent: String,
+    pub project: String,
+    pub signing_key: String,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub encryption_key: Option<String>,
+    #[serde(default)]
+    pub instances: Vec<String>,
+    pub seq: u64,
+    pub by: String,
+    pub at_ms: u64,
+}
+
+#[derive(Debug, Default, Serialize, Deserialize)]
+struct Store {
+    schema: String,
+    cards: BTreeMap<String, Card>,
+}
+
+/// A plain agent or project name: `^[a-z0-9][a-z0-9._-]{0,63}$` and no `..` (design v2, C-M1).
+pub fn valid_name(s: &str) -> bool {
+    let b = s.as_bytes();
+    !b.is_empty()
+        && b.len() <= 64
+        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
+        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'.' | b'_' | b'-'))
+        && !s.contains("..")
+}
+
+fn parse_prefixed_key(s: &str, prefix: &str) -> Option<[u8; 32]> {
+    let hex = s.strip_prefix(prefix)?;
+    if hex.len() != 64 || !hex.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
+        return None;
+    }
+    let mut out = [0u8; 32];
+    for (i, o) in out.iter_mut().enumerate() {
+        *o = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
+    }
+    Some(out)
+}
+
+/// `ed25519-pub:<64 lower hex>` → the 32 key bytes. Compared as bytes, never as strings.
+pub fn parse_signing_key(s: &str) -> Option<[u8; 32]> {
+    parse_prefixed_key(s, SIGNING_PREFIX)
+}
+
+pub fn parse_encryption_key(s: &str) -> Option<[u8; 32]> {
+    parse_prefixed_key(s, ENCRYPTION_PREFIX)
+}
+
+pub fn store_path(dir: &Path) -> PathBuf {
+    dir.join(STORE_FILE)
+}
+
+/// Absent store = no cards. A store that exists but cannot be read or parsed is an `Err`, never
+/// "no cards" (design v2: an unreadable store refuses every gated post).
+fn load(dir: &Path) -> Result<Store, String> {
+    let p = store_path(dir);
+    let text = match std::fs::read_to_string(&p) {
+        Ok(t) => t,
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
+            return Ok(Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() });
+        }
+        Err(e) => return Err(format!("key-card store unreadable: {e}")),
+    };
+    let s: Store = serde_json::from_str(&text).map_err(|e| format!("key-card store corrupt: {e}"))?;
+    if s.schema != STORE_SCHEMA {
+        return Err(format!("key-card store has schema {:?}, not {STORE_SCHEMA}", s.schema));
+    }
+    Ok(s)
+}
+
+fn atomic_write(path: &Path, bytes: &[u8], mode: u32) -> std::io::Result<()> {
+    use std::io::Write;
+    use std::os::unix::fs::OpenOptionsExt;
+    let dir = path.parent().unwrap_or_else(|| Path::new("."));
+    let tmp = dir.join(format!(".{}.tmp.{}", path.file_name().and_then(|n| n.to_str()).unwrap_or("f"), std::process::id()));
+    {
+        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(mode).open(&tmp)?;
+        // exact mode whatever the umask (T-3366 lesson)
+        f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(mode))?;
+        f.write_all(bytes)?;
+        f.sync_all()?;
+    }
+    std::fs::rename(&tmp, path)?;
+    if let Ok(d) = std::fs::File::open(dir) {
+        let _ = d.sync_all();
+    }
+    Ok(())
+}
+
+fn save(dir: &Path, s: &Store) -> std::io::Result<()> {
+    let text = serde_json::to_string_pretty(s).map_err(std::io::Error::other)?;
+    atomic_write(&store_path(dir), text.as_bytes(), 0o600)
+}
+
+/// The outcome of the post-time check.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum Check {
+    /// Not an arc-011 type on `inbox:*`: the check does not apply.
+    NotGated,
+    Pass { agent: String },
+    Refuse { code: i64, fcw: &'static str, agent: String, reason: String },
+}
+
+fn refuse_no_card(agent: &str, reason: String) -> Check {
+    Check::Refuse { code: error_code::CHANNEL_NO_KEY_CARD, fcw: "FCW-135", agent: agent.to_string(), reason }
+}
+
+/// The claimed agent of an arc-011 payload: its `from_agent` (msg and stage records both carry it).
+pub fn claimed_agent(payload: &[u8]) -> Option<String> {
+    let v: serde_json::Value = serde_json::from_slice(payload).ok()?;
+    v.get("from_agent")?.as_str().map(str::to_string)
+}
+
+/// The card check for one post whose transport signature has already verified with `signer`.
+pub fn check_post(dir: &Path, topic: &str, msg_type: &str, payload: &[u8], signer: &[u8; 32]) -> Check {
+    if !topic.starts_with("inbox:") || !GATED_TYPES.contains(&msg_type) {
+        return Check::NotGated;
+    }
+    let agent = match claimed_agent(payload) {
+        Some(a) if valid_name(&a) => a,
+        Some(a) => {
+            let shown: String = a.chars().take(64).collect();
+            return refuse_no_card("?", format!("from_agent {shown:?} is not a plain agent name"));
+        }
+        None => return refuse_no_card("?", "the payload names no from_agent".into()),
+    };
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
+    let store = match load(dir) {
+        Ok(s) => s,
+        Err(e) => return refuse_no_card(&agent, e),
+    };
+    let Some(card) = store.cards.get(&agent) else {
+        return refuse_no_card(&agent, format!("no key card for agent {agent:?}"));
+    };
+    match parse_signing_key(&card.signing_key) {
+        Some(k) if &k == signer => Check::Pass { agent },
+        Some(_) => Check::Refuse {
+            code: error_code::CHANNEL_KEY_CARD_MISMATCH,
+            fcw: "FCW-133",
+            agent: agent.clone(),
+            reason: format!("the post is not signed by the key on agent {agent:?}'s card"),
+        },
+        None => refuse_no_card(&agent, format!("agent {agent:?}'s card holds no valid signing key")),
+    }
+}
+
+/// A card write request (from `hub.key_card_put`).
+#[derive(Debug, Clone, Deserialize)]
+pub struct PutRequest {
+    pub agent: String,
+    pub project: String,
+    pub signing_key: String,
+    #[serde(default)]
+    pub encryption_key: Option<String>,
+    #[serde(default)]
+    pub instances: Vec<String>,
+    #[serde(default)]
+    pub replace: bool,
+}
+
+/// Write or replace one card. Returns the new card and the one it replaced. A change of an
+/// existing card's signing key or project needs `replace: true`, so it is never silent.
+pub fn put(dir: &Path, req: &PutRequest, now_ms: u64) -> Result<(Card, Option<Card>), String> {
+    if !valid_name(&req.agent) {
+        return Err(format!("agent {:?} is not a plain name", req.agent));
+    }
+    if !valid_name(&req.project) {
+        return Err(format!("project {:?} is not a plain name", req.project));
+    }
+    if parse_signing_key(&req.signing_key).is_none() {
+        return Err(format!("signing_key must be {SIGNING_PREFIX}<64 lower hex>"));
+    }
+    if let Some(e) = &req.encryption_key
+        && parse_encryption_key(e).is_none()
+    {
+        return Err(format!("encryption_key must be {ENCRYPTION_PREFIX}<64 lower hex>"));
+    }
+    if let Some(bad) = req.instances.iter().find(|i| !valid_name(i)) {
+        return Err(format!("instance {bad:?} is not a plain name"));
+    }
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
+    let mut store = load(dir)?;
+    let old = store.cards.get(&req.agent).cloned();
+    if let Some(o) = &old
+        && (o.signing_key != req.signing_key || o.project != req.project)
+        && !req.replace
+    {
+        return Err(format!(
+            "agent {:?} already has a card (project {:?}, key {}); changing its key or project needs replace: true",
+            req.agent, o.project, o.signing_key
+        ));
+    }
+    let card = Card {
+        schema: CARD_SCHEMA.into(),
+        agent: req.agent.clone(),
+        project: req.project.clone(),
+        signing_key: req.signing_key.clone(),
+        encryption_key: req.encryption_key.clone(),
+        instances: req.instances.clone(),
+        seq: old.as_ref().map_or(1, |o| o.seq + 1),
+        by: "runme".into(),
+        at_ms: now_ms,
+    };
+    store.schema = STORE_SCHEMA.into();
+    store.cards.insert(card.agent.clone(), card.clone());
+    save(dir, &store).map_err(|e| format!("key-card store not written: {e}"))?;
+    Ok((card, old))
+}
+
+pub fn get(dir: &Path, agent: &str) -> Result<Option<Card>, String> {
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
+    Ok(load(dir)?.cards.get(agent).cloned())
+}
+
+pub fn list(dir: &Path) -> Result<Vec<Card>, String> {
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
+    Ok(load(dir)?.cards.into_values().collect())
+}
+
+/// AL-33 2.17.f(2): compare each card's `seq` with the highest `seq` the admin log records for a
+/// `key-card-put` of that agent. A card ahead of the log (a crash between persist and log, or a
+/// card written outside `hub.key_card_put`) or an unreadable store is reported; the caller raises
+/// attention. Known limit: an admin log pruned past a card's last put reports that card too.
+pub fn startup_reconcile(dir: &Path, admin_log: &Path) -> Vec<String> {
+    let cards = match list(dir) {
+        Ok(c) => c,
+        Err(e) => return vec![e],
+    };
+    if cards.is_empty() {
+        return vec![];
+    }
+    let mut logged: BTreeMap<String, u64> = BTreeMap::new();
+    if let Ok(text) = std::fs::read_to_string(admin_log) {
+        for v in text.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()) {
+            if v.get("event").and_then(|e| e.as_str()) == Some("key-card-put")
+                && let (Some(a), Some(s)) = (v.get("agent").and_then(|a| a.as_str()), v.get("seq").and_then(|s| s.as_u64()))
+            {
+                let e = logged.entry(a.to_string()).or_insert(0);
+                *e = (*e).max(s);
+            }
+        }
+    }
+    cards
+        .iter()
+        .filter(|c| logged.get(&c.agent).copied().unwrap_or(0) != c.seq)
+        .map(|c| {
+            format!(
+                "card {} has seq {} but the admin log's last key-card-put for it shows {}",
+                c.agent,
+                c.seq,
+                logged.get(&c.agent).map_or("none".to_string(), |s| s.to_string())
+            )
+        })
+        .collect()
+}
+
+/// Refusals per claimed agent since hub start (the impersonated agent's sidecar reads these,
+/// design v2 C-12), and would-be refusals in `log` mode.
+static REFUSALS: Mutex<BTreeMap<String, (u64, u64)>> = Mutex::new(BTreeMap::new());
+
+/// Count one refusal (`enforced`) or one log-mode would-refuse for `agent`.
+pub fn note_refusal(agent: &str, enforced: bool) {
+    let mut m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
+    let e = m.entry(agent.to_string()).or_insert((0, 0));
+    if enforced { e.0 += 1 } else { e.1 += 1 }
+}
+
+/// `{agent: {refused, would_refuse}}`.
+pub fn refusal_counts() -> serde_json::Value {
+    let m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
+    serde_json::Value::Object(
+        m.iter()
+            .map(|(a, (r, w))| (a.clone(), serde_json::json!({"refused": r, "would_refuse": w})))
+            .collect(),
+    )
+}
+
+/// Admin rows for refusals are coalesced per (claimed agent, poster key, minute) so a token
+/// holder cannot flood the admin log (design v2, C-12). True = write a row for this one.
+static LOGGED: Mutex<(u64, Option<HashSet<(String, String)>>)> = Mutex::new((0, None));
+
+pub fn first_in_minute(agent: &str, signer_fp: &str, now_ms: u64) -> bool {
+    let minute = now_ms / 60_000;
+    let mut g = LOGGED.lock().unwrap_or_else(|p| p.into_inner());
+    if g.0 != minute || g.1.is_none() {
+        *g = (minute, Some(HashSet::new()));
+    }
+    g.1.as_mut().map(|s| s.insert((agent.to_string(), signer_fp.to_string()))).unwrap_or(true)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn key_hex(b: u8) -> String {
+        format!("{SIGNING_PREFIX}{}", format!("{b:02x}").repeat(32))
+    }
+    fn msg_payload(agent: &str) -> Vec<u8> {
+        serde_json::json!({"v": 1, "kind": "msg", "from_agent": agent, "body": "x"}).to_string().into_bytes()
+    }
+    fn put_req(agent: &str, project: &str, key: u8) -> PutRequest {
+        PutRequest {
+            agent: agent.into(),
+            project: project.into(),
+            signing_key: key_hex(key),
+            encryption_key: None,
+            instances: vec![],
+            replace: false,
+        }
+    }
+
+    #[test]
+    fn names_are_plain_or_refused() {
+        for ok in ["fx-a", "claude-termlink", "a", "010-termlink", "a.b_c-d"] {
+            assert!(valid_name(ok), "{ok}");
+        }
+        for bad in ["", "../../tmp/x", "a..b", "A", "-a", ".a", "a/b", "a b", &"a".repeat(65)] {
+            assert!(!valid_name(bad), "{bad:?}");
+        }
+    }
+
+    #[test]
+    fn mode_defaults_off_and_a_bad_setting_enforces() {
+        let d = tempfile::tempdir().unwrap();
+        assert_eq!(read_mode(d.path()), Ok(Mode::Off));
+        write_mode(d.path(), Mode::Log).unwrap();
+        assert_eq!(read_mode(d.path()), Ok(Mode::Log));
+        std::fs::write(d.path().join(MODE_FILE), "maybe\n").unwrap();
+        assert!(read_mode(d.path()).is_err());
+        assert_eq!(effective_mode(d.path()), Mode::Enforce, "an invalid setting fails closed");
+    }
+
+    #[test]
+    fn e169_fcw135_a_poster_without_a_card_is_refused() {
+        let d = tempfile::tempdir().unwrap();
+        match check_post(d.path(), "inbox:h/fx-b", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]) {
+            Check::Refuse { code, fcw, agent, .. } => {
+                assert_eq!((code, fcw, agent.as_str()), (error_code::CHANNEL_NO_KEY_CARD, "FCW-135", "fx-a"));
+            }
+            o => panic!("{o:?}"),
+        }
+    }
+
+    #[test]
+    fn e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes() {
+        let d = tempfile::tempdir().unwrap();
+        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        put(d.path(), &put_req("fx-b", "p", 2), 1).unwrap();
+        // fx-b's key claiming to be fx-a
+        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[2; 32]) {
+            Check::Refuse { code, fcw, .. } => assert_eq!((code, fcw), (error_code::CHANNEL_KEY_CARD_MISMATCH, "FCW-133")),
+            o => panic!("{o:?}"),
+        }
+        assert_eq!(
+            check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]),
+            Check::Pass { agent: "fx-a".into() }
+        );
+    }
+
+    #[test]
+    fn r75e_another_projects_agent_cannot_post_as_ps_agent() {
+        let d = tempfile::tempdir().unwrap();
+        put(d.path(), &put_req("p-agent", "p", 1), 1).unwrap();
+        put(d.path(), &put_req("q-agent", "q", 2), 1).unwrap();
+        // q's session, signing with its own key, asserting it is p's agent
+        assert!(matches!(
+            check_post(d.path(), "inbox:h/x", "sidecar.stage.v1", &msg_payload("p-agent"), &[2; 32]),
+            Check::Refuse { fcw: "FCW-133", .. }
+        ));
+    }
+
+    #[test]
+    fn a_traversal_name_is_refused_and_never_read_as_a_path() {
+        let d = tempfile::tempdir().unwrap();
+        std::fs::write(d.path().join("x.json"), "{}").unwrap();
+        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("../x"), &[1; 32]) {
+            Check::Refuse { fcw: "FCW-135", agent, .. } => assert_eq!(agent, "?"),
+            o => panic!("{o:?}"),
+        }
+    }
+
+    #[test]
+    fn other_types_and_topics_are_never_gated() {
+        let d = tempfile::tempdir().unwrap();
+        for (topic, ty) in [
+            ("inbox:h/x", "sidecar.receipt"),
+            ("inbox:h/x", "chat"),
+            ("liveness:h/x", "sidecar.liveness.v1"),
+            ("framework:pickup", "sidecar.msg.v1"),
+        ] {
+            assert_eq!(check_post(d.path(), topic, ty, b"{}", &[1; 32]), Check::NotGated, "{topic} {ty}");
+        }
+    }
+
+    #[test]
+    fn an_unreadable_store_refuses_never_reads_as_empty() {
+        let d = tempfile::tempdir().unwrap();
+        std::fs::write(store_path(d.path()), "{not json").unwrap();
+        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]) {
+            Check::Refuse { fcw: "FCW-135", reason, .. } => assert!(reason.contains("corrupt"), "{reason}"),
+            o => panic!("{o:?}"),
+        }
+        assert!(put(d.path(), &put_req("fx-a", "p", 1), 1).is_err(), "a put never overwrites a corrupt store");
+    }
+
+    #[test]
+    fn a_key_or_project_change_needs_replace_and_seq_rises() {
+        let d = tempfile::tempdir().unwrap();
+        let (c1, old) = put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        assert_eq!((c1.seq, old), (1, None));
+        // same key and project: an idempotent re-put moves seq, no replace needed
+        let (c2, _) = put(d.path(), &put_req("fx-a", "p", 1), 2).unwrap();
+        assert_eq!(c2.seq, 2);
+        let e = put(d.path(), &put_req("fx-a", "p", 9), 3).unwrap_err();
+        assert!(e.contains("replace: true"), "{e}");
+        let e = put(d.path(), &put_req("fx-a", "q", 1), 3).unwrap_err();
+        assert!(e.contains("replace: true"), "{e}");
+        let mut r = put_req("fx-a", "p", 9);
+        r.replace = true;
+        let (c3, old) = put(d.path(), &r, 4).unwrap();
+        assert_eq!(c3.seq, 3);
+        assert_eq!(old.unwrap().signing_key, key_hex(1));
+    }
+
+    #[test]
+    fn the_store_is_0600_and_keys_are_validated() {
+        use std::os::unix::fs::PermissionsExt;
+        let d = tempfile::tempdir().unwrap();
+        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        assert_eq!(std::fs::metadata(store_path(d.path())).unwrap().permissions().mode() & 0o777, 0o600);
+        let mut r = put_req("fx-b", "p", 1);
+        r.signing_key = "ed25519:".to_string() + &"01".repeat(32);
+        assert!(put(d.path(), &r, 1).is_err(), "the fingerprint form is not a card key");
+        r.signing_key = key_hex(1).to_uppercase();
+        assert!(put(d.path(), &r, 1).is_err());
+    }
+
+    #[test]
+    fn al33_startup_reconcile_flags_a_card_ahead_of_the_admin_log() {
+        let d = tempfile::tempdir().unwrap();
+        let log = d.path().join("rpc-admin.jsonl");
+        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        put(d.path(), &put_req("fx-b", "p", 2), 1).unwrap();
+        // fx-a's put was logged; fx-b's crashed between persist and log
+        std::fs::write(&log, "{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"seq\":1}\n").unwrap();
+        let m = startup_reconcile(d.path(), &log);
+        assert_eq!(m.len(), 1, "{m:?}");
+        assert!(m[0].contains("fx-b") && m[0].contains("none"), "{m:?}");
+        std::fs::write(&log, "{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"seq\":1}\n{\"event\":\"key-card-put\",\"agent\":\"fx-b\",\"seq\":1}\n").unwrap();
+        assert!(startup_reconcile(d.path(), &log).is_empty());
+        std::fs::write(store_path(d.path()), "{x").unwrap();
+        assert_eq!(startup_reconcile(d.path(), &log).len(), 1, "an unreadable store is reported");
+    }
+
+    #[test]
+    fn refusal_rows_are_coalesced_per_agent_key_minute() {
+        assert!(first_in_minute("t-agent", "fp1", 120_000));
+        assert!(!first_in_minute("t-agent", "fp1", 120_500));
+        assert!(first_in_minute("t-agent", "fp2", 120_600));
+        assert!(first_in_minute("t-agent", "fp1", 180_000), "a new minute logs again");
+    }
+}
diff --git a/crates/termlink-hub/src/lib.rs b/crates/termlink-hub/src/lib.rs
index 12fcfb97c..3e69e71bf 100644
--- a/crates/termlink-hub/src/lib.rs
+++ b/crates/termlink-hub/src/lib.rs
@@ -9,6 +9,7 @@ pub mod dispatch_contract;
 pub mod governor;
 pub mod hub_identity;
 pub mod inbox;
+pub mod key_card;
 pub mod mail_guard;
 pub mod pidfile;
 pub mod remote_store;
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index cfc920062..0e8122c2e 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -168,6 +168,10 @@ pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse
             crate::artifact::handle_artifact_get(id, &req.params).await
         }
         "hub.version" => handle_hub_version(id),
+        "hub.key_card_put" => handle_key_card_put(id, &req.params, peer_addr),
+        "hub.key_card_mode" => handle_key_card_mode(id, &req.params, peer_addr),
+        "hub.key_card_get" => handle_key_card_get(id, &req.params),
+        "hub.key_card_list" => handle_key_card_list(id),
         "hub.legacy_usage" => handle_hub_legacy_usage(id, &req.params),
         "hub.bus_state" => handle_hub_bus_state(id),
         "hub.governor_status" => handle_hub_governor_status(id),
@@ -891,6 +895,117 @@ fn handle_register_remote(id: serde_json::Value, params: &serde_json::Value) ->
 /// T-3366 (AL-17 invariant 3): `hub_id` is the minted `hub.id` the hub read at start, never
 /// recomputed from the certificate (so a rotation renames nothing); `signing_key` is
 /// `ed25519:<sha256 of the public key>` (AL-18). Both null only before the hub has started.
+/// T-3704 (B-6): key-card writes are accepted only over the local unix socket (no peer address;
+/// operator ruling A 2026-11-18). They are admin RPCs, so the intent row is already written.
+fn key_card_local_only(id: &serde_json::Value, method: &str, peer_addr: Option<&str>) -> Option<RpcResponse> {
+    peer_addr.map(|p| {
+        ErrorResponse::with_data(
+            id.clone(),
+            control::error_code::KEY_CARD_REFUSED,
+            &format!("refused: {method} is accepted only over the hub's local unix socket, not from {p}"),
+            serde_json::json!({"method": method}),
+        )
+        .into()
+    })
+}
+
+fn key_card_refused(id: serde_json::Value, reason: &str) -> RpcResponse {
+    ErrorResponse::with_data(
+        id,
+        control::error_code::KEY_CARD_REFUSED,
+        &format!("refused: {reason}"),
+        serde_json::json!({}),
+    )
+    .into()
+}
+
+fn handle_key_card_put(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    if let Some(r) = key_card_local_only(&id, "hub.key_card_put", peer_addr) {
+        return r;
+    }
+    let req: crate::key_card::PutRequest = match serde_json::from_value(params.clone()) {
+        Ok(r) => r,
+        Err(e) => return key_card_refused(id, &format!("bad params: {e}")),
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    let now = std::time::SystemTime::now()
+        .duration_since(std::time::UNIX_EPOCH)
+        .map(|d| d.as_millis() as u64)
+        .unwrap_or(0);
+    match crate::key_card::put(&dir, &req, now) {
+        Ok((card, old)) => {
+            // Every put raises attention (operator ruling A: visible, not impossible); the row
+            // names old and new key so a silent replacement cannot happen.
+            let row = serde_json::json!({
+                "event": "key-card-put", "attention": true, "agent": card.agent, "project": card.project,
+                "seq": card.seq, "signing_key": card.signing_key,
+                "old_signing_key": old.as_ref().map(|o| o.signing_key.clone()),
+                "old_project": old.as_ref().map(|o| o.project.clone()), "replace": req.replace,
+            });
+            if let Err(e) = crate::rpc_audit::admin_note(row) {
+                tracing::error!(error = %e, "T-3704: key-card put admin row not written");
+            }
+            Response::success(id, serde_json::json!({"ok": true, "card": card, "replaced": old})).into()
+        }
+        Err(e) => key_card_refused(id, &e),
+    }
+}
+
+fn handle_key_card_mode(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    let dir = termlink_session::discovery::runtime_dir();
+    let Some(want) = params.get("mode").and_then(|v| v.as_str()) else {
+        let cur = crate::key_card::read_mode(&dir);
+        return Response::success(
+            id,
+            serde_json::json!({"mode": cur.as_ref().map(|m| m.as_str()).unwrap_or("enforce"), "error": cur.err()}),
+        )
+        .into();
+    };
+    if let Some(r) = key_card_local_only(&id, "hub.key_card_mode", peer_addr) {
+        return r;
+    }
+    let Some(mode) = crate::key_card::Mode::parse(want) else {
+        return key_card_refused(id, &format!("mode {want:?} is not off|log|enforce"));
+    };
+    let before = crate::key_card::read_mode(&dir).map(|m| m.as_str()).unwrap_or("invalid");
+    if let Err(e) = crate::key_card::write_mode(&dir, mode) {
+        return key_card_refused(id, &format!("setting not written: {e}"));
+    }
+    let row = serde_json::json!({"event": "key-card-mode", "attention": true, "from": before, "to": mode.as_str()});
+    if let Err(e) = crate::rpc_audit::admin_note(row) {
+        tracing::error!(error = %e, "T-3704: key-card mode admin row not written");
+    }
+    Response::success(id, serde_json::json!({"ok": true, "mode": mode.as_str(), "was": before})).into()
+}
+
+fn handle_key_card_get(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
+    let Some(agent) = params.get("agent").and_then(|v| v.as_str()) else {
+        return key_card_refused(id, "missing 'agent'");
+    };
+    match crate::key_card::get(&termlink_session::discovery::runtime_dir(), agent) {
+        Ok(card) => Response::success(id, serde_json::json!({"card": card})).into(),
+        Err(e) => key_card_refused(id, &e),
+    }
+}
+
+fn handle_key_card_list(id: serde_json::Value) -> RpcResponse {
+    let dir = termlink_session::discovery::runtime_dir();
+    let mode = crate::key_card::read_mode(&dir);
+    match crate::key_card::list(&dir) {
+        Ok(cards) => Response::success(
+            id,
+            serde_json::json!({
+                "mode": mode.as_ref().map(|m| m.as_str()).unwrap_or("enforce"),
+                "mode_error": mode.err(),
+                "cards": cards,
+                "refusals": crate::key_card::refusal_counts(),
+            }),
+        )
+        .into(),
+        Err(e) => key_card_refused(id, &e),
+    }
+}
+
 fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
     let instance = crate::tls::own_cert_fingerprint();
     let ident = crate::hub_identity::current();
@@ -1150,6 +1265,10 @@ fn handle_hub_capabilities(id: serde_json::Value) -> RpcResponse {
         control::method::ARTIFACT_PUT,
         control::method::ARTIFACT_GET,
         "hub.version",
+        "hub.key_card_put",
+        "hub.key_card_mode",
+        "hub.key_card_get",
+        "hub.key_card_list",
         "hub.legacy_usage",
         "hub.bus_state",
         "hub.governor_status",
@@ -2222,6 +2341,24 @@ mod tests {
     use serde_json::json;
     use std::path::{Path, PathBuf};
 
+    #[test]
+    fn t3704_key_card_writes_are_refused_off_the_local_socket() {
+        let put = json!({"agent": "fx-a", "project": "p", "signing_key": format!("ed25519-pub:{}", "01".repeat(32))});
+        for (resp, m) in [
+            (handle_key_card_put(json!(1), &put, Some("<ip>:5000")), "hub.key_card_put"),
+            (handle_key_card_mode(json!(1), &json!({"mode": "off"}), Some("<ip>:5000")), "hub.key_card_mode"),
+        ] {
+            match resp {
+                RpcResponse::Error(e) => {
+                    assert_eq!(e.error.code, control::error_code::KEY_CARD_REFUSED, "{m}");
+                    assert!(e.error.message.contains("local unix socket"), "{m}: {}", e.error.message);
+                }
+                RpcResponse::Success(_) => panic!("{m} accepted over TCP"),
+            }
+        }
+        assert!(crate::rpc_audit::is_admin_method("hub.key_card_put") && crate::rpc_audit::is_admin_method("hub.key_card_mode"));
+    }
+
     /// A scratch root that is writable and NOT under /tmp (the volatile-dir
     /// heuristic must not fire). T-3677: it sits in this build's own target dir
     /// (`<target-dir>/<profile>/test-tmp`, from the test binary's location), not
diff --git a/crates/termlink-hub/src/rpc_audit.rs b/crates/termlink-hub/src/rpc_audit.rs
index 659bbc451..65632d23d 100644
--- a/crates/termlink-hub/src/rpc_audit.rs
+++ b/crates/termlink-hub/src/rpc_audit.rs
@@ -617,6 +617,9 @@ pub const ADMIN_METHODS: &[&str] = &[
     termlink_protocol::control::method::CHANNEL_DELETE,
     termlink_protocol::control::method::CHANNEL_FORCE_RELEASE,
     termlink_protocol::control::method::CHANNEL_TRANSFER_CLAIM,
+    // T-3704 (B-6): card writes and the check setting (admin row before the act, FCW-128).
+    "hub.key_card_put",
+    "hub.key_card_mode",
 ];
 
 pub fn is_admin_method(method: &str) -> bool {
@@ -715,6 +718,16 @@ fn admin_append(row: &serde_json::Value) -> std::io::Result<()> {
     Ok(())
 }
 
+/// T-3704: one admin-log event row that is not an RPC's intent/result pair (a key-card refusal,
+/// a card put). `ts` and `admin: true` are added here.
+pub(crate) fn admin_note(mut row: serde_json::Value) -> std::io::Result<()> {
+    if let Some(o) = row.as_object_mut() {
+        o.insert("ts".into(), serde_json::json!(now_ms() as u64));
+        o.insert("admin".into(), serde_json::json!(true));
+    }
+    admin_append(&row)
+}
+
 /// FCW-128: write the intent row for an admin RPC. `Err` → the caller refuses
 /// the RPC. Returns the row id the result row refers to.
 pub fn admin_intent(method: &str, caller: &AdminCaller<'_>, params: &serde_json::Value) -> std::io::Result<String> {
diff --git a/crates/termlink-hub/src/server.rs b/crates/termlink-hub/src/server.rs
index 3dc58dae7..bd075e4ec 100644
--- a/crates/termlink-hub/src/server.rs
+++ b/crates/termlink-hub/src/server.rs
@@ -607,6 +607,15 @@ pub async fn run_with_tcp(
     // for T-1166 entry-gate measurement.
     crate::rpc_audit::init(&runtime_dir);
     crate::circuit_breaker::init(&runtime_dir);
+    // T-3704 (AL-33 2.17.f(2)): a card whose seq the admin log does not show (a crash between
+    // persist and log, or an edit outside hub.key_card_put) raises an attention row at start.
+    for m in crate::key_card::startup_reconcile(&runtime_dir, &runtime_dir.join(crate::rpc_audit::ADMIN_FILE_NAME)) {
+        tracing::warn!(detail = %m, "T-3704: key-card seq not in the admin log");
+        let row = serde_json::json!({"event": "key-card-seq-mismatch", "attention": true, "detail": m});
+        if let Err(e) = crate::rpc_audit::admin_note(row) {
+            tracing::error!(error = %e, "T-3704: key-card seq mismatch row not written");
+        }
+    }
 
     // T-2048: Install connection-cap + per-sender rate-limit governors.
     // Reads TERMLINK_MAX_CONNECTIONS / TERMLINK_RATE_LIMIT_PER_SEC from env,
@@ -771,7 +780,12 @@ fn hub_method_scope(method: &str) -> PermissionScope {
         | control::method::EVENT_COLLECT
         | "hub.version"
         | "hub.bus_state"
-        | "hub.governor_status" => PermissionScope::Observe,
+        | "hub.governor_status"
+        | "hub.key_card_get"
+        | "hub.key_card_list" => PermissionScope::Observe,
+
+        // T-3704 (B-6): card writes and the check setting; also local-socket only (router).
+        "hub.key_card_put" | "hub.key_card_mode" => PermissionScope::Execute,
 
         // Observe: read-only channel/bus + agent introspection (T-2267).
         // Before this arm existed the entire channel.* surface fell through
diff --git a/crates/termlink-protocol/src/control.rs b/crates/termlink-protocol/src/control.rs
index 2e0629a23..6c5585b7b 100644
--- a/crates/termlink-protocol/src/control.rs
+++ b/crates/termlink-protocol/src/control.rs
@@ -563,6 +563,15 @@ pub mod error_code {
     /// written, at the session or at the hub. Nothing was done. `data` carries
     /// `{fcw: "FCW-132"}`. (-32040/-32041 are the hub's mail-guard codes.)
     pub const USE_LOG_UNWRITABLE: i64 = -32042;
+    /// T-3704 (arc-011 B-6, FCW-135): an arc-011 post (`sidecar.msg.v1` / `sidecar.stage.v1` on
+    /// `inbox:*`) names an agent with no key card at the hub, names no valid agent, or the card
+    /// store cannot be read. `data` carries `{fcw, agent}`.
+    pub const CHANNEL_NO_KEY_CARD: i64 = -32043;
+    /// T-3704 (FCW-133): the post is signed by a key other than the claimed agent's card key.
+    pub const CHANNEL_KEY_CARD_MISMATCH: i64 = -32044;
+    /// T-3704: a key-card write was refused (not over the local socket, bad name or key, or a
+    /// change of an existing card's key or project without `replace`).
+    pub const KEY_CARD_REFUSED: i64 = -32045;
 }
 
 /// Default `protocol_version` when the field is missing on the wire.
diff --git a/crates/termlink-sidecar/src/daemon.rs b/crates/termlink-sidecar/src/daemon.rs
index 7a5924aa8..1d4b8e3dd 100644
--- a/crates/termlink-sidecar/src/daemon.rs
+++ b/crates/termlink-sidecar/src/daemon.rs
@@ -109,14 +109,28 @@ impl Drop for Running {
 
 fn roster_of(keys: &[String]) -> Result<Roster, String> {
     let mut r = Roster::new();
-    for k in keys {
+    for entry in keys {
+        // T-3704 (B-6): `agent=<hex>` binds the key to that agent name; bare `<hex>` stays unbound.
+        let (name, k) = match entry.split_once('=') {
+            Some((n, k)) => {
+                if !crate::wire::plain_agent_name(n) {
+                    return Err(format!("roster entry names {n:?}, not a plain agent name"));
+                }
+                (Some(n), k)
+            }
+            None => (None, entry.as_str()),
+        };
         let bytes: Vec<u8> = (0..k.len())
             .step_by(2)
             .map(|i| k.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok()))
             .collect::<Option<_>>()
             .ok_or_else(|| format!("roster key is not hex: {k}"))?;
         let arr: [u8; 32] = bytes.try_into().map_err(|_| format!("roster key is not 32 bytes: {k}"))?;
-        r.add(VerifyingKey::from_bytes(&arr).map_err(|e| format!("roster key: {e}"))?);
+        let pk = VerifyingKey::from_bytes(&arr).map_err(|e| format!("roster key: {e}"))?;
+        match name {
+            Some(n) => r.bind(n, pk),
+            None => r.add(pk),
+        };
     }
     Ok(r)
 }
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index f2364da41..fca74de63 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -53,6 +53,8 @@ pub const HELD_ALERT: &str = "held-for-operator";
 pub const HELD_KEY_PREFIX: &str = "held-key-";
 /// The store state file holding the next inbox offset to read.
 pub const CURSOR_STATE: &str = "inbox-cursor";
+/// T-3704: the last refusal count the hub reported for this agent (impersonation attention).
+pub const IMPERSONATION_STATE: &str = "key-card-refusals-seen";
 /// Conversation name used for a refused message whose own conv is unusable.
 pub const UNPARSED_CONV: &str = "_refused";
 
@@ -253,6 +255,44 @@ impl Inner {
         }
     }
 
+    /// T-3704 (B-6, design v2 C-12): the hub counts arc-011 posts refused (or, in log mode, that
+    /// would be refused) for claiming to be this agent. When that count has grown since the last
+    /// tick, raise one needs-attention entry per day naming the counts. Best effort: an old hub
+    /// without `hub.key_card_list` or any read error changes nothing and never holds mail.
+    fn observe_impersonation(&self) {
+        let Ok(resp) = self.rpc("hub.key_card_list", json!({})) else { return };
+        let Ok(v) = unwrap_result(resp) else { return };
+        let r = &v["refusals"][self.cfg.agent.as_str()];
+        let (refused, would) = (r["refused"].as_u64().unwrap_or(0), r["would_refuse"].as_u64().unwrap_or(0));
+        let total = refused + would;
+        let seen: u64 = self
+            .store
+            .read_state(IMPERSONATION_STATE)
+            .ok()
+            .flatten()
+            .and_then(|s| s.trim().parse().ok())
+            .unwrap_or(0);
+        if total == seen {
+            return;
+        }
+        let _ = self.store.write_state(IMPERSONATION_STATE, &total.to_string());
+        if total < seen {
+            return; // the hub restarted and its counts began again
+        }
+        let day = std::time::SystemTime::now()
+            .duration_since(std::time::UNIX_EPOCH)
+            .map(|d| d.as_secs() / 86_400)
+            .unwrap_or(0);
+        let detail = format!(
+            "posts claiming to be agent {} were refused at the hub by its key card: {refused} refused, \
+             {would} would be refused (log mode); see `termlink hub key-card list` and rpc-admin.jsonl",
+            self.cfg.agent
+        );
+        if let Err(e) = self.store.attention_coalesced("posts-claiming-to-be-you-refused", &day.to_string(), &detail) {
+            eprintln!("termlink-sidecar: needs attention (posts-claiming-to-be-you-refused): {detail} [could not record it: {e}]");
+        }
+    }
+
     fn cursor(&self) -> u64 {
         self.store
             .read_state(CURSOR_STATE)
@@ -288,7 +328,10 @@ impl Inner {
             return PassResult::Drained;
         }
         match self.check_pin() {
-            Ok(()) => mem.held = None,
+            Ok(()) => {
+                mem.held = None;
+                self.observe_impersonation();
+            }
             Err(PinError::Unreachable(_)) => return PassResult::Waiting,
             Err(PinError::Mismatch { declared, found }) => {
                 let why = format!(
diff --git a/crates/termlink-sidecar/src/wire.rs b/crates/termlink-sidecar/src/wire.rs
index 6a74dedb3..3335b9c31 100644
--- a/crates/termlink-sidecar/src/wire.rs
+++ b/crates/termlink-sidecar/src/wire.rs
@@ -240,6 +240,9 @@ pub enum Refusal {
     BadSignature,
     /// `to_instance` names another or ended instance (SI-26): never redirected.
     WrongInstance { want: String, got: String },
+    /// T-3704 (B-6, FCW-98): the claimed `from_agent` is bound by name to another key in the
+    /// roster, or the signer key is bound to another agent name.
+    AgentKeyMismatch,
 }
 
 impl fmt::Display for Refusal {
@@ -254,6 +257,9 @@ impl fmt::Display for Refusal {
             Self::WrongInstance { want, got } => {
                 write!(f, "message is for instance {got}, this is {want}")
             }
+            Self::AgentKeyMismatch => {
+                write!(f, "the claimed from_agent and the signer key are bound to different agents in the roster")
+            }
         }
     }
 }
@@ -261,8 +267,10 @@ impl fmt::Display for Refusal {
 impl std::error::Error for Refusal {}
 
 /// Sender fingerprint -> public key. PH-1: filled by the harness (note 7.2).
+/// T-3704 (B-6): optionally also agent name -> key (a NAMED entry, `agent=<hex>` in the config),
+/// so a key may only speak as the agent it is bound to. Unnamed entries keep the PH-1 behaviour.
 #[derive(Debug, Clone, Default)]
-pub struct Roster(BTreeMap<String, VerifyingKey>);
+pub struct Roster(BTreeMap<String, VerifyingKey>, BTreeMap<String, VerifyingKey>);
 
 impl Roster {
     pub fn new() -> Self {
@@ -285,11 +293,52 @@ impl Roster {
         for (fp, pk) in &other.0 {
             self.0.insert(fp.clone(), *pk);
         }
+        for (name, pk) in &other.1 {
+            self.1.insert(name.clone(), *pk);
+        }
+    }
+
+    /// T-3704: add a key bound to an agent name; returns its fingerprint.
+    pub fn bind(&mut self, agent: &str, pk: VerifyingKey) -> String {
+        self.1.insert(agent.to_string(), pk);
+        self.add(pk)
+    }
+
+    /// Number of named entries (status: "bound" vs "unbound" senders).
+    pub fn bound_count(&self) -> usize {
+        self.1.len()
+    }
+
+    /// T-3704 (FCW-98): `from_agent` must be a plain name; if it is bound, the signer must be its
+    /// key; and a key bound to some agent may not speak as another one.
+    pub fn check_agent(&self, from_agent: &str, pk: &VerifyingKey) -> Result<(), Refusal> {
+        if !plain_agent_name(from_agent) {
+            return Err(Refusal::Malformed("from_agent is not a plain agent name"));
+        }
+        if let Some(k) = self.1.get(from_agent)
+            && k.as_bytes() != pk.as_bytes()
+        {
+            return Err(Refusal::AgentKeyMismatch);
+        }
+        if self.1.iter().any(|(name, k)| k.as_bytes() == pk.as_bytes() && name != from_agent) {
+            return Err(Refusal::AgentKeyMismatch);
+        }
+        Ok(())
     }
 }
 
+/// `^[a-z0-9][a-z0-9._-]{0,63}$`, no `..` (the hub's key-card name rule, T-3704).
+pub fn plain_agent_name(s: &str) -> bool {
+    let b = s.as_bytes();
+    !b.is_empty()
+        && b.len() <= 64
+        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
+        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'.' | b'_' | b'-'))
+        && !s.contains("..")
+}
+
 /// Signer key -> `from`, roster, signature: the checks shared by messages and stages.
-fn check_signer(s: &Signed, roster: &Roster, from: &str, msg_type: &str) -> Result<(), Refusal> {
+fn check_signer(s: &Signed, roster: &Roster, from: &str, from_agent: &str, msg_type: &str) -> Result<(), Refusal> {
     let pk_bytes: [u8; 32] = unhex(&s.signer_pk)
         .and_then(|b| b.try_into().ok())
         .ok_or(Refusal::Malformed("signer key"))?;
@@ -301,6 +350,7 @@ fn check_signer(s: &Signed, roster: &Roster, from: &str, msg_type: &str) -> Resu
     if rostered.as_bytes() != pk.as_bytes() {
         return Err(Refusal::KeyMismatch);
     }
+    roster.check_agent(from_agent, &pk)?;
     let sig_bytes: [u8; 64] = unhex(&s.sig)
         .and_then(|b| b.try_into().ok())
         .ok_or(Refusal::Malformed("signature"))?;
@@ -320,7 +370,7 @@ pub fn verify(s: &Signed, roster: &Roster, my_instance: &str) -> Result<Msg, Ref
     if msg.payload_bytes() != s.payload {
         return Err(Refusal::NotCanonical);
     }
-    check_signer(s, roster, &msg.from, MSG_TYPE)?;
+    check_signer(s, roster, &msg.from, &msg.from_agent, MSG_TYPE)?;
     if msg.to_instance != my_instance {
         return Err(Refusal::WrongInstance {
             want: my_instance.to_owned(),
@@ -361,6 +411,8 @@ pub fn verify_open(s: &Signed, roster: &Roster, my_instance: &str) -> Result<(Ms
     let mut own = Roster::new();
     own.add(pk);
     let msg = verify(s, &own, my_instance)?;
+    // T-3704: an unrostered key may still not claim an agent name the roster binds to another key.
+    roster.check_agent(&msg.from_agent, &pk)?;
     Ok((msg, Signer::Unrostered))
 }
 
@@ -374,6 +426,7 @@ pub fn refusal_code(r: &Refusal) -> &'static str {
         Refusal::KeyMismatch => "key-mismatch",
         Refusal::BadSignature => "bad-signature",
         Refusal::WrongInstance { .. } => "wrong-instance",
+        Refusal::AgentKeyMismatch => "agent-key-mismatch",
     }
 }
 
@@ -474,7 +527,7 @@ pub fn verify_stage(s: &Signed, roster: &Roster) -> Result<Stage, Refusal> {
     if st.payload_bytes() != s.payload {
         return Err(Refusal::NotCanonical);
     }
-    check_signer(s, roster, &st.from, STAGE_TYPE)?;
+    check_signer(s, roster, &st.from, &st.from_agent, STAGE_TYPE)?;
     Ok(st)
 }
 
@@ -769,6 +822,49 @@ mod wire_tests {
         assert!(matches!(verify(&s, &r, "inst-b"), Err(Refusal::Malformed(_))));
     }
 
+    /// T-3704 (B-6, E-69.4 receiver half, FCW-98): with both keys rostered, a post claiming
+    /// `alice` signed by bob's key is refused once the roster binds names; the bound key passes.
+    #[test]
+    fn t3704_named_roster_binds_from_agent_to_its_key() {
+        let (alice, bob) = (ident(11), ident(12));
+        let mut r = Roster::new();
+        r.bind("alice", alice.verifying_key());
+        r.bind("bob", bob.verifying_key());
+        // bob's key, message says from_agent "alice" (msg_from always claims alice)
+        let s = sign(&bob, "inbox:b", 42, &msg_from(&bob, "inst-b"));
+        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::AgentKeyMismatch));
+        assert_eq!(refusal_code(&Refusal::AgentKeyMismatch), "agent-key-mismatch");
+        // alice's own key passes
+        let ok = sign(&alice, "inbox:b", 42, &msg_from(&alice, "inst-b"));
+        assert!(verify(&ok, &r, "inst-b").is_ok());
+        // a bound key speaking as an unbound name is refused too
+        let mut m = msg_from(&bob, "inst-b");
+        m.from_agent = "carol".into();
+        let s = sign(&bob, "inbox:b", 42, &m);
+        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::AgentKeyMismatch));
+        // an unrostered key claiming a bound name is refused by verify_open as well
+        let eve = ident(13);
+        let s = sign(&eve, "inbox:b", 42, &msg_from(&eve, "inst-b"));
+        assert_eq!(verify_open(&s, &r, "inst-b").map(|_| ()), Err(Refusal::AgentKeyMismatch));
+        // a traversal-style name is refused before any lookup
+        let mut m = msg_from(&alice, "inst-b");
+        m.from_agent = "../x".into();
+        let s = sign(&alice, "inbox:b", 42, &m);
+        assert!(matches!(verify(&s, &r, "inst-b"), Err(Refusal::Malformed(_))));
+    }
+
+    /// T-3704: unnamed (legacy) roster entries keep the PH-1 behaviour: no name binding.
+    #[test]
+    fn t3704_unnamed_roster_entries_stay_unbound() {
+        let (alice, bob) = (ident(14), ident(15));
+        let mut r = Roster::new();
+        r.add(alice.verifying_key());
+        r.add(bob.verifying_key());
+        assert_eq!(r.bound_count(), 0);
+        let s = sign(&bob, "inbox:b", 42, &msg_from(&bob, "inst-b"));
+        assert!(verify(&s, &r, "inst-b").is_ok(), "unbound: from_agent is not checked against the key");
+    }
+
     #[test]
     fn wire_roster_key_mismatch_refused() {
         // Roster holds a different key under A's fingerprint slot.
```

## Test: crates/termlink-cli/tests/t3704_key_card.rs
```rust
//! T-3704 (arc-011 P-16, B-6): E-69 post half and E-169's card case at L4 — a real hub binary in a
//! throwaway runtime dir. Cards are written with `termlink hub key-card put` (the runme path);
//! posts are signed `channel.post` RPCs over the hub socket, so the hub sees exactly what a
//! sidecar (or a forger holding hub access) would send.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use base64::Engine;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn rpc(sock: &Path, method: &str, params: serde_json::Value) -> serde_json::Value {
    let mut s = UnixStream::connect(sock).expect("connect hub");
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    s.write_all(format!("{req}\n").as_bytes()).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("json reply")
}

fn start(dir: &TestDir, tag: &str) -> ProcessGuard {
    let log = |n: &str| std::fs::OpenOptions::new().create(true).append(true).open(dir.path.join(n)).unwrap();
    let child = cmd(dir, &["hub", "start"]).stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

/// A signed arc-011 post: `from_agent` claimed in the payload, signed by `key`.
fn post(sock: &Path, key: &Identity, from_agent: &str, msg_type: &str) -> serde_json::Value {
    let topic = "inbox:0000000000000000/fx-target";
    let payload = serde_json::json!({"v": 1, "kind": "msg", "from_agent": from_agent, "body": "hello"}).to_string();
    let ts = 1_800_000_000_000i64;
    let bytes = canonical_sign_bytes(topic, msg_type, payload.as_bytes(), None, ts);
    rpc(
        sock,
        "channel.post",
        serde_json::json!({
            "topic": topic, "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
            "ts": ts, "sender_id": key.fingerprint(), "sender_pubkey_hex": key.public_key_hex(),
            "signature_hex": key.sign_hex(&bytes),
        }),
    )
}

fn code(v: &serde_json::Value) -> Option<i64> {
    v.get("error").and_then(|e| e.get("code")).and_then(|c| c.as_i64())
}

fn put(dir: &TestDir, agent: &str, project: &str, key: &Identity, replace: bool) -> std::process::Output {
    let k = format!("ed25519-pub:{}", key.public_key_hex());
    let mut a = vec!["hub", "key-card", "put", "--agent", agent, "--project", project, "--signing-key", &k, "--json"];
    if replace {
        a.push("--replace");
    }
    cmd(dir, &a).output().unwrap()
}

#[test]
fn e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused() {
    let dir = TestDir::new("t3704-card");
    let _g = start(&dir, "t3704-card-hub");
    let sock = dir.path.join("hub.sock");
    let (a, b, c) = (Identity::generate(), Identity::generate(), Identity::generate());
    let created = rpc(&sock, "channel.create", serde_json::json!({"name": "inbox:0000000000000000/fx-target"}));
    assert!(created.get("result").is_some(), "{created}");

    // default off: a forgery still passes (the check is not on yet), proving the default.
    assert_eq!(code(&post(&sock, &b, "fx-a", "sidecar.msg.v1")), None, "default off");

    assert!(put(&dir, "fx-a", "proj-p", &a, false).status.success());
    assert!(put(&dir, "fx-b", "proj-p", &b, false).status.success());
    let m = cmd(&dir, &["hub", "key-card", "mode", "enforce"]).output().unwrap();
    assert!(m.status.success(), "{}", String::from_utf8_lossy(&m.stderr));

    // E-69.3: post as fx-a signed with fx-b's key -> refused (FCW-133)
    assert_eq!(code(&post(&sock, &b, "fx-a", "sidecar.msg.v1")), Some(-32044));
    // a holder of hub access with its own fresh key claiming fx-a (BP-3) -> refused
    assert_eq!(code(&post(&sock, &c, "fx-a", "sidecar.stage.v1")), Some(-32044));
    // E-169: an agent with no card -> refused (FCW-135)
    assert_eq!(code(&post(&sock, &c, "fx-z", "sidecar.msg.v1")), Some(-32043));
    // a traversal name never reaches a lookup -> refused (FCW-135)
    assert_eq!(code(&post(&sock, &c, "../x", "sidecar.msg.v1")), Some(-32043));
    // fx-a's own key -> stored
    let ok = post(&sock, &a, "fx-a", "sidecar.msg.v1");
    assert!(ok.get("result").is_some(), "{ok}");
    // another type on inbox:* is never gated (NID)
    assert!(post(&sock, &c, "fx-a", "chat").get("result").is_some());

    // a key change without --replace is refused; with --replace accepted and logged with both keys
    let r = put(&dir, "fx-a", "proj-p", &c, false);
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("replace"), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(put(&dir, "fx-a", "proj-p", &c, true).status.success());
    let admin = std::fs::read_to_string(dir.path.join("rpc-admin.jsonl")).unwrap_or_default();
    assert!(
        admin.lines().any(|l| l.contains("\"key-card-put\"") && l.contains("\"attention\":true")
            && l.contains(&a.public_key_hex()) && l.contains(&c.public_key_hex())),
        "admin log lacks the replace row naming old and new key:\n{admin}"
    );
    assert!(admin.contains("\"key-card-refused\""), "refusals are admin-logged:\n{admin}");

    // per-agent refusal counts for the impersonated agent
    let list = rpc(&sock, "hub.key_card_list", serde_json::json!({}));
    let refused = list["result"]["refusals"]["fx-a"]["refused"].as_u64().unwrap_or(0);
    assert!(refused >= 2, "{list}");

    // log mode: a forgery is stored but counted as would-refuse
    assert!(cmd(&dir, &["hub", "key-card", "mode", "log"]).status().unwrap().success());
    assert!(post(&sock, &b, "fx-a", "sidecar.msg.v1").get("result").is_some(), "log mode stores");
    let list = rpc(&sock, "hub.key_card_list", serde_json::json!({}));
    assert!(list["result"]["refusals"]["fx-a"]["would_refuse"].as_u64().unwrap_or(0) >= 1, "{list}");
}

#[test]
fn e169_a_corrupt_card_store_refuses_every_gated_post() {
    let dir = TestDir::new("t3704-corrupt");
    let _g = start(&dir, "t3704-corrupt-hub");
    let sock = dir.path.join("hub.sock");
    let a = Identity::generate();
    assert!(put(&dir, "fx-a", "proj-p", &a, false).status.success());
    assert!(cmd(&dir, &["hub", "key-card", "mode", "enforce"]).status().unwrap().success());
    std::fs::write(dir.path.join("key-cards.json"), "{broken").unwrap();
    let v = post(&sock, &a, "fx-a", "sidecar.msg.v1");
    assert_eq!(code(&v), Some(-32043), "{v}");
    assert!(v["error"]["message"].as_str().unwrap_or("").contains("corrupt"), "{v}");
    // an invalid setting fails closed, too
    std::fs::write(dir.path.join("key-card-check"), "maybe\n").unwrap();
    std::fs::remove_file(dir.path.join("key-cards.json")).unwrap();
    assert_eq!(code(&post(&sock, &a, "fx-a", "sidecar.msg.v1")), Some(-32043), "no card + invalid setting = enforce");
}
```

## Test: crates/termlink-sidecar/tests/t3704_impersonation.rs
```rust
//! T-3704 (arc-011 P-16, B-6; design v2 C-12): the agent another session tries to impersonate
//! learns of it. A forged arc-011 post claiming `fx-a` is refused at the hub by the key-card check;
//! fx-a's own sidecar reads the hub's per-agent refusal count on its receive tick and raises one
//! needs-attention entry, once per day, and holds no mail for it.

use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use serde_json::{Value, json};
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::Identity;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

fn kinds(store: &Store) -> Vec<(String, String)> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| (v["kind"].as_str().unwrap_or("").to_string(), v["detail"].as_str().unwrap_or("").to_string()))
        .collect()
}

#[test]
fn a_refused_post_claiming_to_be_me_raises_one_attention_entry() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    std::fs::write(e.runtime_dir().join("key-card-check"), "enforce\n").unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "fx-a").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: "fx-a".into(),
                instance: "inst-a".into(),
                identity: e.identity("a").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join("outbox-a.sqlite"),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let rx = Receiver::new(
        ReceiveConfig {
            agent: "fx-a".into(),
            instance: "inst-a".into(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            roster: Roster::new(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender,
        Box::new(BuiltIn::new(Roster::new())),
    )
    .unwrap();

    // no forgery yet: a pass raises nothing
    let _ = rx.pass();
    assert!(!kinds(&store).iter().any(|(k, _)| k == "posts-claiming-to-be-you-refused"));

    // a forger with its own key posts as fx-a; the hub has no card for fx-a and refuses
    let topic = format!("inbox:{id}/fx-b");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    let forger = Identity::generate();
    let payload = json!({"v": 1, "kind": "msg", "from_agent": "fx-a", "body": "forged"}).to_string();
    let ts = 1_800_000_000_000i64;
    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.msg.v1", payload.as_bytes(), None, ts));
    let refused = rpc(
        &e,
        "channel.post",
        json!({"topic": topic, "msg_type": "sidecar.msg.v1",
               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
               "ts": ts, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
               "signature_hex": sig}),
    );
    assert_eq!(refused["error"]["code"], -32043, "{refused}");

    let _ = rx.pass();
    let hits: Vec<_> = kinds(&store).into_iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").collect();
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].1.contains("fx-a") && hits[0].1.contains("1 refused"), "{}", hits[0].1);

    // the next pass with no new refusal adds nothing
    let _ = rx.pass();
    assert_eq!(kinds(&store).iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").count(), 1);
}
```
