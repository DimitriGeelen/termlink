# T-3804 — The 91 orphan inboxes on .107 (2026-11-16)

Found by AEF T-5149 (be.52) after vendoring (T-3802). Metadata only; the first line of a post was read only to judge whether a request was pending. Nothing forwarded or deleted.

## Classes

- **Test residue, 81:** `t3684b-*` 43, `t3684d-*` 34, `t3782a-*`, `t3693a-*`, `e2e-*`, `smk-b`, inboxes of throwaway agents from our own e2e runs (T-3684, T-3693, T-3782). Acknowledged.
- **Our arc-011 W1 sidecar traffic, 2:** `claude-termlink` (8), `claude-termlink-alt` (8): `sidecar.stage.v1`/`sidecar.msg.v1` from the W1 instance ids. Acknowledged.
- **Real mail on a nickname or wrong-hub address, 8 topics.** Each sender was notified with the real circuit (2026-11-16):
  - `aef-greenfield-test` (19): ring20-manager `ntb-002-onedev-repo` (8, told ring20); ours `tl-132-hub-recovery` (10, moot, greenfield back); 055 `scoring-estate` (1, told 055)
  - `proxmox-ring20-management` under .107's hub id (14): 832 (12, told 832 the real circuit 22c19fedafd73da2/…; ring20 told)
  - `claude-shared-toolkit` (12): AEF release notices (9), 055 INCIDENT on worker t1516-onedev (told 055; likely never seen by its owner), ours (2, broadcasts)
  - `agentic-engineering-framework` (1): a consult from 0909 to AEF (told AEF)
  - `ring20-dashboard` (1): from AEF (told AEF the real circuit 1389a831016c4bf1/ring20-dashboard)
  - `nachlauf-c2` (18), `t2336-zwei-stellen` (10), `entlastung-e1b` (7): 1409 to ended workers (told 1409)

Notices: 832 14cef150, ring20 368fb4f9, 055 40e1e188, AEF 06b6ab13, 1409 3c0df245.

## Full table (topic, count, nickname_of, senders, types, first, last, conversations)

```
aef-greenfield-test	19	-	010-termlink:10;proxmox-ring20-management:8;055-agentic-fleet-cockpit:1	sidecar.consult:19	?	?	3
agentic-engineering-framework	1	999-Agentic-Engineering-Framework	0909-boot-into-timeshift-snapshots:1	sidecar.consult:1	?	?	1
claude-shared-toolkit	12	-	999-Agentic-Engineering-Framework:9;010-termlink:2;055-agentic-fleet-cockpit:1	sidecar.consult:11;note:1	?	?	6
claude-termlink	8	-	84c04584:8	sidecar.stage.v1:7;sidecar.msg.v1:1	?	?	2
claude-termlink-alt	8	-	6738c073:8	sidecar.stage.v1:6;sidecar.msg.v1:2	?	?	2
e2e-1fc080c8-sender	8	-	055-agentic-fleet-cockpit:8	sidecar.consult:8	?	?	1
entlastung-e1b	7	-	1409-sprind:7	sidecar.consult:7	?	?	1
nachlauf-c2	18	-	1409-sprind:18	sidecar.consult:18	?	?	1
proxmox-ring20-management	14	-	832-Workflow-designer:12;proxmox-ring20-management:1;1409-sprind:1	sidecar.consult:12;note:2	?	?	1
ring20-dashboard	1	-	999-Agentic-Engineering-Framework:1	sidecar.consult:1	?	?	1
smk-b	10	-	999-Agentic-Engineering-Framework:9;smk-a:1	sidecar.consult:10	?	?	5
t2336-zwei-stellen	10	-	1409-sprind:10	sidecar.consult:10	?	?	1
t3684b-0gl096	4	-	t3684a-0gl096:4	sidecar.consult:4	?	?	1
t3684b-1lquu7	4	-	t3684a-1lquu7:4	sidecar.consult:4	?	?	1
t3684b-1u8ki0	4	-	t3684a-1u8ki0:4	sidecar.consult:4	?	?	1
t3684b-23f7gu	2	-	t3684a-23f7gu:2	sidecar.consult:2	?	?	1
t3684b-4vtzxl	4	-	t3684a-4vtzxl:4	sidecar.consult:4	?	?	1
t3684b-5bo71t	4	-	t3684a-5bo71t:4	sidecar.consult:4	?	?	1
t3684b-6e749w	3	-	t3684a-6e749w:3	sidecar.consult:3	?	?	1
t3684b-6v3wey	1	-	t3684a-6v3wey:1	sidecar.consult:1	?	?	1
t3684b-6z99on	4	-	t3684a-6z99on:4	sidecar.consult:4	?	?	1
t3684b-7hh4s0	4	-	t3684a-7hh4s0:4	sidecar.consult:4	?	?	1
t3684b-7k7cmh	1	-	t3684a-7k7cmh:1	sidecar.consult:1	?	?	1
t3684b-9my9jl	4	-	t3684a-9my9jl:4	sidecar.consult:4	?	?	1
t3684b-b0ddxd	4	-	t3684a-b0ddxd:4	sidecar.consult:4	?	?	1
t3684b-bwec9d	4	-	t3684a-bwec9d:4	sidecar.consult:4	?	?	1
t3684b-d0zgi6	2	-	t3684a-d0zgi6:2	sidecar.consult:2	?	?	1
t3684b-d7wxyv	4	-	t3684a-d7wxyv:4	sidecar.consult:4	?	?	1
t3684b-do47wh	1	-	t3684a-do47wh:1	sidecar.consult:1	?	?	1
t3684b-g068oe	4	-	t3684a-g068oe:4	sidecar.consult:4	?	?	1
t3684b-guxa6x	3	-	t3684a-guxa6x:3	sidecar.consult:3	?	?	1
t3684b-i4a8fw	4	-	t3684a-i4a8fw:4	sidecar.consult:4	?	?	1
t3684b-ie2ayf	3	-	t3684a-ie2ayf:3	sidecar.consult:3	?	?	1
t3684b-iob565	4	-	t3684a-iob565:4	sidecar.consult:4	?	?	1
t3684b-jsmblk	4	-	t3684a-jsmblk:4	sidecar.consult:4	?	?	1
t3684b-kazvld	2	-	t3684a-kazvld:2	sidecar.consult:2	?	?	1
t3684b-l8wf5q	2	-	t3684a-l8wf5q:2	sidecar.consult:2	?	?	1
t3684b-meumcx	4	-	t3684a-meumcx:4	sidecar.consult:4	?	?	1
t3684b-oi3hq9	3	-	t3684a-oi3hq9:3	sidecar.consult:3	?	?	1
t3684b-ow1gmp	4	-	t3684a-ow1gmp:4	sidecar.consult:4	?	?	1
t3684b-ox90o8	2	-	t3684a-ox90o8:2	sidecar.consult:2	?	?	1
t3684b-qb8blu	4	-	t3684a-qb8blu:4	sidecar.consult:4	?	?	1
t3684b-r0801d	4	-	t3684a-r0801d:4	sidecar.consult:4	?	?	1
t3684b-trrfep	4	-	t3684a-trrfep:4	sidecar.consult:4	?	?	1
t3684b-tuslku	4	-	t3684a-tuslku:4	sidecar.consult:4	?	?	1
t3684b-u5mypx	4	-	t3684a-u5mypx:4	sidecar.consult:4	?	?	1
t3684b-uxohzi	4	-	t3684a-uxohzi:4	sidecar.consult:4	?	?	1
t3684b-v9y9jp	2	-	t3684a-v9y9jp:2	sidecar.consult:2	?	?	1
t3684b-vu86y0	4	-	t3684a-vu86y0:4	sidecar.consult:4	?	?	1
t3684b-woejcj	2	-	t3684a-woejcj:2	sidecar.consult:2	?	?	1
t3684b-x4kec5	2	-	t3684a-x4kec5:2	sidecar.consult:2	?	?	1
t3684b-y6bf6o	4	-	t3684a-y6bf6o:4	sidecar.consult:4	?	?	1
t3684b-yg6jak	4	-	t3684a-yg6jak:4	sidecar.consult:4	?	?	1
t3684b-z86d3k	4	-	t3684a-z86d3k:4	sidecar.consult:4	?	?	1
t3684b-zkuc8n	4	-	t3684a-zkuc8n:4	sidecar.consult:4	?	?	1
t3684d-0gl096	1	-	t3684a-0gl096:1	sidecar.consult:1	?	?	1
t3684d-1u8ki0	1	-	t3684a-1u8ki0:1	sidecar.consult:1	?	?	1
t3684d-23f7gu	1	-	t3684a-23f7gu:1	sidecar.consult:1	?	?	1
t3684d-4vtzxl	1	-	t3684a-4vtzxl:1	sidecar.consult:1	?	?	1
t3684d-5bo71t	1	-	t3684a-5bo71t:1	sidecar.consult:1	?	?	1
t3684d-6e749w	1	-	t3684a-6e749w:1	sidecar.consult:1	?	?	1
t3684d-6v3wey	1	-	t3684a-6v3wey:1	sidecar.consult:1	?	?	1
t3684d-6z99on	1	-	t3684a-6z99on:1	sidecar.consult:1	?	?	1
t3684d-7hh4s0	1	-	t3684a-7hh4s0:1	sidecar.consult:1	?	?	1
t3684d-bwec9d	1	-	t3684a-bwec9d:1	sidecar.consult:1	?	?	1
t3684d-d7wxyv	1	-	t3684a-d7wxyv:1	sidecar.consult:1	?	?	1
t3684d-do47wh	1	-	t3684a-do47wh:1	sidecar.consult:1	?	?	1
t3684d-g068oe	1	-	t3684a-g068oe:1	sidecar.consult:1	?	?	1
t3684d-i4a8fw	1	-	t3684a-i4a8fw:1	sidecar.consult:1	?	?	1
t3684d-iob565	1	-	t3684a-iob565:1	sidecar.consult:1	?	?	1
t3684d-j0t321	1	-	t3684a-j0t321:1	sidecar.consult:1	?	?	1
t3684d-jsmblk	1	-	t3684a-jsmblk:1	sidecar.consult:1	?	?	1
t3684d-l8wf5q	1	-	t3684a-l8wf5q:1	sidecar.consult:1	?	?	1
t3684d-lvxesg	1	-	t3684a-lvxesg:1	sidecar.consult:1	?	?	1
t3684d-meumcx	1	-	t3684a-meumcx:1	sidecar.consult:1	?	?	1
t3684d-ow1gmp	1	-	t3684a-ow1gmp:1	sidecar.consult:1	?	?	1
t3684d-qb8blu	1	-	t3684a-qb8blu:1	sidecar.consult:1	?	?	1
t3684d-r0801d	1	-	t3684a-r0801d:1	sidecar.consult:1	?	?	1
t3684d-trrfep	1	-	t3684a-trrfep:1	sidecar.consult:1	?	?	1
t3684d-tuslku	1	-	t3684a-tuslku:1	sidecar.consult:1	?	?	1
t3684d-u5mypx	1	-	t3684a-u5mypx:1	sidecar.consult:1	?	?	1
t3684d-uxohzi	1	-	t3684a-uxohzi:1	sidecar.consult:1	?	?	1
t3684d-vu86y0	1	-	t3684a-vu86y0:1	sidecar.consult:1	?	?	1
t3684d-x4kec5	1	-	t3684a-x4kec5:1	sidecar.consult:1	?	?	1
t3684d-y6bf6o	1	-	t3684a-y6bf6o:1	sidecar.consult:1	?	?	1
t3684d-yblka3	1	-	t3684a-yblka3:1	sidecar.consult:1	?	?	1
t3684d-yg6jak	1	-	t3684a-yg6jak:1	sidecar.consult:1	?	?	1
t3684d-z86d3k	1	-	t3684a-z86d3k:1	sidecar.consult:1	?	?	1
t3684d-zkuc8n	1	-	t3684a-zkuc8n:1	sidecar.consult:1	?	?	1
t3693a-7gybgt	1	-	t3693b-7gybgt:1	sidecar.receipt:1	?	?	1
t3782a-99x8ur	1	-	t3782b-99x8ur:1	sidecar.receipt:1	?	?	1
```
