# SQ-19 — `check-receiver-ack-lag` red: who is not acking, and what to do

Decision brief for the operator. Prepared 2026-09-30 under T-3211. Read-only investigation;
nothing was acked, posted or changed.

## Background

`scripts/check-receiver-ack-lag.sh` is a **WARN**-tier guard member. The WARN ledger
(`.context/checks/guard-warn-first-red`) records it red since **2026-09-29T22:04:59Z**, so on
about **2026-10-13 22:05 UTC** the release canary's filer will open a task for it automatically.

What a row means, read from the code (`compute_ack_status`, `crates/termlink-cli/src/commands/channel.rs` ~8628):

- The rows come from `channel ack-status <topic>` on the **local hub**. A row exists for every
  identity fingerprint that has **posted anything** on the topic, or posted a receipt.
- `NEVER-ACKED <fp>` means **that fingerprint has never posted a receipt on that topic**. It is
  the party that did not ack, not the sender of the unread messages.
- For a NEVER-ACKED row, `lag` is not a gap. It is `latest_content_offset + 1`, roughly the
  topic's total content count, **including the identity's own posts**.
- Scope: every `dm:*` topic plus `ACK_LAG_INCLUDE_TOPICS`. Broadcast rails are excluded.
  **No exclusion knob exists.** There is no env var and no allowlist. The only way to narrow the
  scope is `--topics`, which replaces the whole set.

**Two design gaps follow from this, and they matter for the decision:**
1. **A sender that only posts gets flagged.** If an identity is the sole poster on a topic, it
   has nothing inbound to acknowledge, but it still reads NEVER-ACKED.
2. **A recipient that never posts is invisible.** The party that actually fails to read never
   appears as a row. Example: `dm:0e7ee6…:6a646c…` has only our three `release` posts, and the
   real recipient has no row at all.

## Who the fingerprints are

| fp | Identity | Evidence |
|---|---|---|
| `d1993c2c3ec44c94` | **This host (dimitrimintdev, .107).** It is the default `/root/.termlink/identity.key`, shared by every agent here. | `termlink agent identity --json` → `fingerprint d1993c2c3ec44c94`; T-3004 F4 |
| `9219671e28054458` | **ring20-management (.122) host identity**, i.e. `ring20-management-agent`. It is mostly an unattended poster (presence notes, T-1438 heartbeats). T-3004 F3 calls it stale. It acks rarely: it did ack `dm:050-email-archive:…` (lag 0). | T-3004 F3; T-1898 ("zero receipts from 9219671e → no attached claude reading"); T-1425 table |
| `33df8954b2a9b70d` | **ring20-dashboard (.121)**, `ring20-dashboard-agent`. | T-1989 episodic ("self-fp 33df8954b2a9b70d") |

## Evidence: the 11 NEVER-ACKED rows (live run 2026-09-30, rc=1, 19 dm topics scanned)

Content was taken from `channel subscribe --cursor 0 --json`; receipts are excluded.

| # | Topic | Row fp | lag | Posters (content) | Class |
|---|---|---|---|---|---|
| 1 | dm:3bba15e6…:6738c073… | d1993 (us) | 146 | 3bba×10, 84c0×11, us×1 (Sep 22) | **a** — we never acked peer posts |
| 2 | dm:61e262f0…:9219671e… | d1993 (us) | 94 | 9219×93, us×1 (Sep 3–25) | **a** — 93 inbound from .122 unacked |
| 3 | dm:61e262f0…:9219671e… | 9219 (.122) | 94 | same | **b** — .122 never acked our 1 post |
| 4 | dm:9219671e…:d1993c2c3ec44c94 | 9219 (.122) | 39 | 9219×10, us×12 (to today); we are lag 0 | **b** |
| 5 | dm:cashweb-integration-agent:d1993… | 9219 (.122) | 7 | 9219×2, us×3; we are lag 0 | **b** |
| 6 | dm:050-email-archive:ring20-management-agent | d1993 (us) | 1 | us×1 only (recipient 9219 acked) | **c** — sole sender |
| 7 | dm:0e7ee6ca…:6a646ce8… | d1993 (us) | 3 | us×3 only (`release`) | **c** — sole sender; recipient invisible |
| 8 | dm:33df8954…:8e6fd77e… | 33df (.121) | 2 | 33df×2 only, Sep 3 | **c** — sole sender, dead since Sep 3 |
| 9 | dm:3bba15e6…:9219671e… | 9219 (.122) | 3 | 9219×3 only, Sep 16–19 | **c** — sole sender |
| 10 | dm:9219671e…:d1993c2c | 9219 (.122) | 5 | 9219×5 only, Sep 6 | **c** — sole sender, malformed topic (truncated fp) |
| 11 | dm:sprin-d:ring20-management-agent | 9219 (.122) | 3 | 9219×3 only, Sep 25 | **c** — sole sender |

**Counts: a = 2, b = 3, c = 6.** In 6 of 11 rows the flagged identity is the only poster, so it
had nothing to ack. Those rows are artifacts of the check's design, not missed messages. Of the
5 real rows, 2 are ours and 3 belong to .122, which is also the identity behind 4 of the 6
artifact rows.

## Options

| Opt | Action | Effect on the red | Cost / risk |
|---|---|---|---|
| **A** | Ack our own backlog now: `termlink channel ack` on rows 1, 2, 6, 7, which are signed as d1993. | Clears 4 rows. **Still red** on the 7 rows owned by .122 and .121. | Minutes. Rows 1–2 need someone to actually read 94 + 22 posts first, or the ack just "clears the gauge". A standing ack-on-read policy for our consumers is a separate, larger change. |
| **B** | Add an exclusion ledger, `.context/checks/receiver-ack-lag-allowlist` (`<topic>  # reason`), in the pattern of the other git-tracked ledgers. List dead/malformed topics 8 and 10 and the .122 bot topics. | Could clear everything, but only by hiding rows. | A code change, because no knob exists today. It risks acknowledging real peer silence (rows 3–5) as "fine". |
| **E** | **Fix the predicate.** Emit NEVER-ACKED only when the identity has inbound content, meaning at least one content post by another sender after its first post. Also count the missing-recipient case (gap 2) as its own honest class. | Removes the 6 class-c artifacts and keeps the 5 real rows. | A small change to the script plus fixtures (the classifier is already a single function). It changes what the canary claims, so it needs the self-test extended. |
| **C** | Reclassify the member as INFO. | Never escalates. | Silences the only receiver-side signal. The 5 real rows remain. |
| **D** | Leave it red and let 2026-10-13 file the task. | The filed task inherits this analysis. | No work now. 6 of 11 findings are noise, so the task starts out misframed. |

## Recommendation

Do **E + A**, and do not do B or C.
- **E first.** It is the root cause. More than half the red is the check flagging senders for not
  acking their own messages. The same shape was already fixed once on broadcast rails (T-3256,
  the fatigue class).
- **Then A** for rows 1 and 2, after someone actually reads the 94 .122 posts and the 3bba thread.
- The remaining 3 rows (b) are a real finding about **.122**: `ring20-management-agent` mostly
  does not ack DMs. That is a peer or comms-policy matter, for ring20-manager scope. File it as a
  handoff. It is not a local exclusion.

With E and A done, the member drops to 3 rows, all owned by .122. It stays red until .122 starts
acking or you decide those DMs need no reply. That is the honest state.

## What you must decide

1. **Predicate change (E):** may a build task narrow NEVER-ACKED to identities that have inbound
   content, and report "recipient never posted" separately? (yes / no)
2. **Our backlog (A):** should an agent read and ack rows 1–2 as this host's identity, or should
   they stay unacked until a human has read the .122 traffic? (agent / human / leave)
3. **.122 non-acking (rows 3–5):** is it a peer fix (hand off to ring20-management), or is DM
   acking not required of unattended bot identities? The second answer implies B for those
   topics, with a cited reason.
4. **Escalation date:** accept the auto-filed task on 2026-10-13 as the tracking vehicle, or file
   now under SQ-19? A manual task pre-empts the auto-filing only if its body carries the
   `<!-- warn-escalation: member=check-receiver-ack-lag -->` de-dup marker.

## Outcome (2026-09-30)

**Operator ruling:** "as recommended", i.e. E (fix the predicate) + A (an agent acks our backlog) + hand-off of the .122 rows + a tracking task filed now (T-3277, which carries the de-dup marker).

**E, built in T-3277.** Live re-run after the fix, compared with the 11-row baseline above:

| Baseline row | New class | Fires? |
|---|---|---|
| 1, 2 (us, lag 146 / 94) | NEVER-ACKED | yes (class a, our backlog) |
| 3, 4, 5 (.122) | NEVER-ACKED | yes (class b, peer) |
| 6, 7 (us, sole sender) | SOLE-SENDER | no |
| 8 (.121, sole sender) | SOLE-SENDER | no |
| 9, 10, 11 (.122, sole sender) | SOLE-SENDER | no |

7 SOLE-SENDER rows are counted, not fired: the baseline's 6 plus one more .121 sole-sender topic seen in this run. The new RECIPIENT-SILENT class lists 8 named parties with no row. It is **reported, not fired**, because topic names often carry a per-agent fp while the posts are signed with the host's shared key (T-3004 F4). Firing would recreate the fatigue this fix removes. Firing rows: 11 → 5, exactly classes a + b.

**Hand-off to .122 (T-3279).** Posted on `dm:9219671e28054458:d1993c2c3ec44c94` at offset 60 (read back). The .122 hub was unreachable at the time (fleet probe rc=124), so this is **posted, not confirmed**. It asks ring20-management either to ack its 3 topics (lags 94 / 59 / 7) or to reply with its ack policy for an unattended bot identity.
