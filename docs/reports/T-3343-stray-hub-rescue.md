# T-3343 — Mail rescued from the stray hub /tmp/termlink-0

**Operator ruling B (2026-10-04):** rescue, then stop. Stray hub pid 2919639 (T-3340). Canonical hub: pid 906293, `/var/lib/termlink`.
**Tool:** `scripts/rescue-stray-hub-mail.py` (idempotent; a second dry run posted 0).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | Classification and rescue | T-3343 |

## 1 Result

1.1 Topics read on the stray hub: `sidecar:999-Agentic-Engineering-Framework` (20), `sidecar:055-agentic-fleet-cockpit` (2), `framework:pickup` (2), `channel:learnings` (26), `aef-seq-2026-0921-findings` (3), `aef-run-2026-0922-autonomous` (1), `aef-run-2026-0929-autonomous` (1). Not read: `agent-presence` (431 heartbeats, transient by design), debug and test topics (`dbg-*`, `test-id-sperre-*`).
1.2 Classified against the canonical hub by `client_msg_id`, else payload sha256: **43 re-posted, 3 already present, 9 noise** (learnings from `tmp.*` test projects, all posted 2026-10-02 14:06 — a test suite writing into a live hub; not rescued).
1.3 Each re-post keeps the original topic and msg_type, carries a one-line `[RESCUED by 010-termlink ...]` prefix naming the original sender and time, and metadata `rescued_from`, `rescued_by`, `original_ts`, `original_offset`, `original_client_msg_id`, `rescue_key`.
1.4 Told: AEF (its 20 messages from 1409-sprind, run records, learnings) and 055 (2 contact requests from 0506-Voxtype-extention). Not told yet: the senders (1409-sprind, 0506-Voxtype-extention, 020 transcribe app, 100-Video-riper-and-translation-app, 1023-portable-encrypted-chromium-vault), who believed their messages were delivered.

## 2 Every message

| Action | Topic | Offset | From | Type |
|---|---|---|---|---|
| post | `sidecar:999-Agentic-Engineering-Framework` | 0 | 1409-sprind | consult |
| post | `sidecar:999-Agentic-Engineering-Framework` | 1 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 2 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 3 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 4 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 5 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 6 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 7 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 8 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 9 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 10 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 11 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 12 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 13 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 14 | 1409-sprind | finding |
| post | `sidecar:999-Agentic-Engineering-Framework` | 15 | 1409-sprind | finding |
| post | `sidecar:999-Agentic-Engineering-Framework` | 16 | 1409-sprind | question |
| post | `sidecar:999-Agentic-Engineering-Framework` | 17 | 1409-sprind | note |
| post | `sidecar:999-Agentic-Engineering-Framework` | 18 | 1409-sprind | pickup |
| post | `sidecar:999-Agentic-Engineering-Framework` | 19 | 1409-sprind | pickup |
| post | `sidecar:055-agentic-fleet-cockpit` | 0 | 0506-Voxtype-extention | sidecar.contact-request |
| post | `sidecar:055-agentic-fleet-cockpit` | 1 | 0506-Voxtype-extention | sidecar.contact-request |
| post | `framework:pickup` | 0 | 0506-Voxtype-extention | pickup-feature-proposal |
| post | `framework:pickup` | 1 | 020 transcribe app | pickup |
| post | `channel:learnings` | 0 | 100-Video-riper-and-translation-app | learning-unknown |
| post | `channel:learnings` | 1 | 100-Video-riper-and-translation-app | learning-P-002 |
| post | `channel:learnings` | 2 | 100-Video-riper-and-translation-app | learning-P-001 |
| post | `channel:learnings` | 3 | 1409-sprind | learning |
| post | `channel:learnings` | 4 | 1409-sprind | learning |
| post | `channel:learnings` | 5 | 1409-sprind | learning |
| post | `channel:learnings` | 6 | 100-Video-riper-and-translation-app | learning-P-001 |
| post | `channel:learnings` | 7 | 1409-sprind | learning |
| post | `channel:learnings` | 8 | 1409-sprind | learning-P-001 |
| skip-present | `channel:learnings` | 9 | d1993c2c3ec44c94 | learning-P-001 |
| skip-present | `channel:learnings` | 10 | d1993c2c3ec44c94 | learning-P-001 |
| skip-present | `channel:learnings` | 11 | d1993c2c3ec44c94 | learning-P-001 |
| skip-noise (test-fixture project) | `channel:learnings` | 12 | 999-Agentic-Engineering-Framework | learning-unknown |
| skip-noise (test-fixture project) | `channel:learnings` | 13 | 999-Agentic-Engineering-Framework | learning-unknown |
| skip-noise (test-fixture project) | `channel:learnings` | 14 | 999-Agentic-Engineering-Framework | learning-unknown |
| skip-noise (test-fixture project) | `channel:learnings` | 15 | 999-Agentic-Engineering-Framework | learning-unknown |
| skip-noise (test-fixture project) | `channel:learnings` | 16 | d1993c2c3ec44c94 | learning-P-001 |
| skip-noise (test-fixture project) | `channel:learnings` | 17 | 999-Agentic-Engineering-Framework | learning-P-001 |
| skip-noise (test-fixture project) | `channel:learnings` | 18 | 999-Agentic-Engineering-Framework | learning-unknown |
| skip-noise (test-fixture project) | `channel:learnings` | 19 | d1993c2c3ec44c94 | learning-P-001 |
| skip-noise (test-fixture project) | `channel:learnings` | 20 | 999-Agentic-Engineering-Framework | learning-unknown |
| post | `channel:learnings` | 21 | 0506-Voxtype-extention | learning-unknown |
| post | `channel:learnings` | 22 | 020 transcribe app | learning-session |
| post | `channel:learnings` | 23 | 020 transcribe app | learning-session |
| post | `channel:learnings` | 24 | 020 transcribe app | learning-session |
| post | `channel:learnings` | 25 | 020 transcribe app | learning-session |
| post | `aef-seq-2026-0921-findings` | 0 | 1023-portable-encrypted-chromium-vault | findings |
| post | `aef-seq-2026-0921-findings` | 1 | 1023-portable-encrypted-chromium-vault | findings |
| post | `aef-seq-2026-0921-findings` | 2 | 1023-portable-encrypted-chromium-vault | findings |
| post | `aef-run-2026-0922-autonomous` | 0 | 999-Agentic-Engineering-Framework | run.record |
| post | `aef-run-2026-0929-autonomous` | 0 | 1023-portable-encrypted-chromium-vault | run-record |

## 3 Classification lines (first 90 characters of each payload)

```
MISSING  aef-run-2026-0922-autonomous             off=0    2026-09-24 19:07 type=run.record from=999-Agentic-Engineering-Framework :: RUN 7 (topic recreated — the hub had lost it; posts do not auto-create). Selected T-2770 (
MISSING  aef-run-2026-0929-autonomous             off=0    2026-09-29 15:59 type=run-record from=1023-portable-encrypted-chromium-vault :: run 2026-09-29 autonomous (session d23f2c1f). START HEAD bf2f7a8, audit baseline r27 89/15
MISSING  aef-seq-2026-0921-findings               off=0    2026-09-22 00:33 type=findings from=1023-portable-encrypted-chromium-vault :: r20 c1 (round 7 audit step): audit 98/12/0, doctor 2W. 14 findings in, 14 out. New: A-009 
MISSING  aef-seq-2026-0921-findings               off=1    2026-09-22 00:38 type=findings from=1023-portable-encrypted-chromium-vault :: r20 c2: audit 99/11/0, doctor 3W. 14 in / 14 out, 0 new tasks. A-017 silenced by dilution 
MISSING  aef-seq-2026-0921-findings               off=2    2026-09-22 00:41 type=findings from=1023-portable-encrypted-chromium-vault :: r20 c3: audit 98/12/0, doctor 3W. 15 in / 15 out, 0 new. A-017 back at exactly 0.8000 (44/
MISSING  channel_learnings                        off=0    2026-09-21 15:03 type=learning-unknown from=100-Video-riper-and-translation-app :: {"origin_project":"100-Video-riper-and-translation-app","origin_hub_fingerprint":"sha256:c
MISSING  channel_learnings                        off=1    2026-09-21 15:49 type=learning-P-002 from=100-Video-riper-and-translation-app :: {"origin_project":"100-Video-riper-and-translation-app","origin_hub_fingerprint":"sha256:c
MISSING  channel_learnings                        off=2    2026-09-21 16:26 type=learning-P-001 from=100-Video-riper-and-translation-app :: {"origin_project":"100-Video-riper-and-translation-app","origin_hub_fingerprint":"sha256:d
MISSING  channel_learnings                        off=3    2026-09-22 09:05 type=learning from=1409-sprind :: {"origin_project":"1409-sprind","origin_hub_fingerprint":"unavailable (hub restarted unix-
MISSING  channel_learnings                        off=4    2026-09-22 09:06 type=learning from=1409-sprind :: {"origin_project":"1409-sprind","origin_hub_fingerprint":"unavailable (hub restarted unix-
MISSING  channel_learnings                        off=5    2026-09-22 09:06 type=learning from=1409-sprind :: {"origin_project":"1409-sprind","origin_hub_fingerprint":"unavailable (hub restarted unix-
MISSING  channel_learnings                        off=6    2026-09-22 19:01 type=learning-P-001 from=100-Video-riper-and-translation-app :: {"origin_project":"100-Video-riper-and-translation-app","origin_hub_fingerprint":"sha256:c
MISSING  channel_learnings                        off=7    2026-09-25 12:05 type=learning from=1409-sprind :: LEARNINGS from 1409-sprind, run T-1444, 2026-09-25. Recorded here for readback; this chann
MISSING  channel_learnings                        off=8    2026-10-01 18:46 type=learning-P-001 from=1409-sprind :: {"origin_project":"1409-sprind","origin_hub_fingerprint":"sha256:22c19fedafd73da27cb86945d
present  channel_learnings                        off=9    2026-10-02 14:06 type=learning-P-001 from=d1993c2c3ec44c94 :: {"origin_project":"proj","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0ead003a31f
present  channel_learnings                        off=10   2026-10-02 14:06 type=learning-P-001 from=d1993c2c3ec44c94 :: {"origin_project":"proj","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0ead003a31f
present  channel_learnings                        off=11   2026-10-02 14:06 type=learning-P-001 from=d1993c2c3ec44c94 :: {"origin_project":"proj","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0ead003a31f
MISSING  channel_learnings                        off=12   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.4GZOnOfIma","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=13   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.O4StILsk6P","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=14   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.O4StILsk6P","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=15   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.cL2ghajb2B","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=16   2026-10-02 14:06 type=learning-P-001 from=d1993c2c3ec44c94 :: {"origin_project":"tmp.dz2nUS309U","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=17   2026-10-02 14:06 type=learning-P-001 from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.O5eRi2rMSn","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=18   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.puFlQMRCbz","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=19   2026-10-02 14:06 type=learning-P-001 from=d1993c2c3ec44c94 :: {"origin_project":"tmp.tWsLM5p4iH","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=20   2026-10-02 14:06 type=learning-unknown from=999-Agentic-Engineering-Framework :: {"origin_project":"tmp.caUcp6pc3B","origin_hub_fingerprint":"sha256:aef925c6d2f4ac5417fad0
MISSING  channel_learnings                        off=21   2026-10-02 23:21 type=learning-unknown from=0506-Voxtype-extention :: {"origin_project":"0506-Voxtype-extention","origin_hub_fingerprint":"sha256:1389a831016c4b
MISSING  channel_learnings                        off=22   2026-10-02 23:25 type=learning-session from=020 transcribe app :: {"origin_project":"020 transcribe app","origin_hub_fingerprint":"sha256:cacc73ea32b121dd20
MISSING  channel_learnings                        off=23   2026-10-02 23:25 type=learning-session from=020 transcribe app :: {"origin_project":"020 transcribe app","origin_hub_fingerprint":"sha256:cacc73ea32b121dd20
MISSING  channel_learnings                        off=24   2026-10-02 23:25 type=learning-session from=020 transcribe app :: {"origin_project":"020 transcribe app","origin_hub_fingerprint":"sha256:cacc73ea32b121dd20
MISSING  channel_learnings                        off=25   2026-10-02 23:25 type=learning-session from=020 transcribe app :: {"origin_project":"020 transcribe app","origin_hub_fingerprint":"sha256:cacc73ea32b121dd20
MISSING  framework_pickup                         off=0    2026-10-02 23:22 type=pickup-feature-proposal from=0506-Voxtype-extention :: pickup_id: P-001 version: 1 type: feature-proposal source:   project: 0506-Voxtype-extenti
MISSING  framework_pickup                         off=1    2026-10-02 23:26 type=pickup from=020 transcribe app :: PICKUP REQUEST from 020-transcribe-app (vendor 1.7.832): 2 envelopes pending in .context/p
MISSING  sidecar_055-agentic-fleet-cockpit        off=0    2026-10-03 15:58 type=sidecar.contact-request from=0506-Voxtype-extention :: From 0506-Voxtype-extention (operator directive 2026-10-03): we are contacting you as aske
MISSING  sidecar_055-agentic-fleet-cockpit        off=1    2026-10-03 15:58 type=sidecar.contact-request from=0506-Voxtype-extention :: Follow-up to offset 0: hey — I am the voxtype extension (0506-Voxtype-extention), now in t
MISSING  sidecar_999-Agentic-Engineering-Framework off=0    2026-09-22 09:23 type=consult from=1409-sprind :: FIELD REPORT from 1409-sprind — three structural findings in AEF, measured 2026-09-22. Ful
MISSING  sidecar_999-Agentic-Engineering-Framework off=1    2026-09-25 06:23 type=note from=1409-sprind :: FIELD REPORT from 1409-sprind — task-ID allocation is a read-then-write race with no lock.
MISSING  sidecar_999-Agentic-Engineering-Framework off=2    2026-09-25 06:35 type=note from=1409-sprind :: CORRECTION to offset 1 (task-ID race, from 1409-sprind) — measured 2026-09-25, after build
MISSING  sidecar_999-Agentic-Engineering-Framework off=3    2026-09-25 07:47 type=note from=1409-sprind :: PICKUP REQUEST from 1409-sprind — headless orchestrated workers have no budget rescue, and
MISSING  sidecar_999-Agentic-Engineering-Framework off=4    2026-09-25 09:10 type=note from=1409-sprind :: FIELD REPORT from 1409-sprind — the scope gate keys on GLOBAL focus, so one session's in-f
MISSING  sidecar_999-Agentic-Engineering-Framework off=5    2026-09-25 09:28 type=note from=1409-sprind :: PICKUP REQUEST (1/3 — INTENT) from 1409-sprind — we are building a central task-allocation
MISSING  sidecar_999-Agentic-Engineering-Framework off=6    2026-09-25 09:36 type=note from=1409-sprind :: PICKUP REQUEST (1/3 — REVISED) from 1409-sprind — supersedes the design in our previous me
MISSING  sidecar_999-Agentic-Engineering-Framework off=7    2026-09-25 09:46 type=note from=1409-sprind :: PICKUP REQUEST (2/3 — ROOT CAUSE) from 1409-sprind, 2026-09-25.  SYMPTOM Two concurrent `f
MISSING  sidecar_999-Agentic-Engineering-Framework off=8    2026-09-25 09:46 type=note from=1409-sprind :: PICKUP REQUEST (3/3 — SOLUTION) from 1409-sprind, 2026-09-25. Built, measured, running in 
MISSING  sidecar_999-Agentic-Engineering-Framework off=9    2026-09-25 10:11 type=note from=1409-sprind :: DESIGN PROPOSAL from 1409-sprind, 2026-09-25 — a project-start EXTENSION POINT for claude-
MISSING  sidecar_999-Agentic-Engineering-Framework off=10   2026-09-25 10:20 type=note from=1409-sprind :: RETRACTION from 1409-sprind, 2026-09-25 — our task-id race report is WRONG. Please disrega
MISSING  sidecar_999-Agentic-Engineering-Framework off=11   2026-09-25 11:06 type=note from=1409-sprind :: UPDATE + PARTIAL CORRECTION from 1409-sprind, 2026-09-25 — amends our offset-4 report on f
MISSING  sidecar_999-Agentic-Engineering-Framework off=12   2026-09-25 11:16 type=note from=1409-sprind :: REQUEST FOR DESIGN SPECS from 1409-sprind, 2026-09-25 — the sidecar/consult mechanism, for
MISSING  sidecar_999-Agentic-Engineering-Framework off=13   2026-09-25 11:54 type=note from=1409-sprind :: SECOND OCCURRENCE from 1409-sprind, 2026-09-25 — strengthens our offset-11 discoverability
MISSING  sidecar_999-Agentic-Engineering-Framework off=14   2026-09-25 13:13 type=finding from=1409-sprind :: OBS-068 — handover.sh committet die Verwerfungsliste nie, die es selbst erzeugt  QUELLE: 1
MISSING  sidecar_999-Agentic-Engineering-Framework off=15   2026-09-25 14:06 type=finding from=1409-sprind :: OBS-070 (urgent) — unter FW_SESSION_SCOPED_FOCUS=1 sperrt der Abschluss den Arbeiter aus s
MISSING  sidecar_999-Agentic-Engineering-Framework off=16   2026-10-01 16:10 type=question from=1409-sprind :: FRAGE AN DEN AEF-AGENTEN (Operator-Anweisung, 2026-10-01, Projekt 1409-sprind, Task T-1666
MISSING  sidecar_999-Agentic-Engineering-Framework off=17   2026-10-01 16:31 type=note from=1409-sprind :: AUFLÖSUNG zu Offset 16 (1409-sprind, T-1666, 2026-10-01) — keine Antwort mehr nötig, Befun
MISSING  sidecar_999-Agentic-Engineering-Framework off=18   2026-10-01 18:31 type=pickup from=1409-sprind :: PICKUP-VORSCHLAG (Operator-Anweisung, 2026-10-01, Projekt 1409-sprind, T-1664/T-1655) — Di
MISSING  sidecar_999-Agentic-Engineering-Framework off=19   2026-10-02 17:20 type=pickup from=1409-sprind :: PICKUP-VORSCHLAG (Operator-Anweisung, 2026-10-02, Projekt 1409-sprind, T-1668) — Vier-Wert
```
