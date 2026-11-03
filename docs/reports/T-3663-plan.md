# T-3663 — plan for the remaining file-route findings (for the operator after his trip)

Source: docs/reports/T-3663-file-routes-inventory.md (29 routes + 9 peer answers). Rule applied throughout (operator,
2026-11-03): keep every working route until its replacement is proven in the field; retire only with the operator's
explicit, documented approval. Nothing below has been changed; each item is one decision, in this recommended order.

## Done or decided already
| Finding | State |
|---|---|
| B5 .107 Docker API open on :2375 | FIXED 2026-11-03 15:51Z (T-3664, runme 27) |
| C1 OneDev pushes over plain HTTP | FIXED for 4 projects (050 .107, ring20-mgmt, ring20-dashboard, dimitri-mint-dev); 050's CT 250 (.129) deploy fetch still http, operator deferred |
| C2 WireGuard conf incl. private key over plain HTTP | DEFERRED until the operator is back (T-3665, revisit 2026-11-10; ring20 T-2595 parked, no VPN change) |
| A4 `remote push` world-readable /tmp files + payload in `ps` | Fix in code under T-3667 (live only with the operator's runme install) |

## Open, in recommended order
1. **B4 — .201 Docker API open on :2375 (plain, no auth), used daily to copy the payments tracker DB** (109-anche-payments
   `deploy/backup-remote-db.sh`). Two problems: (a) the open API is root on .201 for anyone on the LAN (same class as B5);
   (b) a payments database crosses the LAN unencrypted. Recommended: A — switch the backup to `docker -H ssh://` (or TLS)
   first and prove it for a few days, then close .201's :2375 with the operator's approval (the host owner, likely
   ring20/.201's operator route, does the close). Severity: high (root-equivalent + financial data).
2. **A1/A2/A8 — files and inline payloads stored in plaintext on hubs** (artifact store, topic store, node1 pool bundles).
   Already decided in principle by O-1 (encrypt-only for the new cross-hub route) and designed in step 4 (B-11 per-agent
   keys + wrapper, B-12 blobs, B-13 `--encrypt-to`). Recommended: no separate decision; follow the step-4 build order,
   then extend `--encrypt-to` to same-hub artifact writers and inline bundles (A8) as a follow-up build item. Keep the
   plaintext routes until the encrypted ones are proven (rule).
3. **C3 — PBS backup replica .122 → NAS over NFS, unencrypted (secret groups excluded)** (ring20's route). Options: A
   PBS client-side encryption for the replica; B keep as is with the exclusion recorded as a risk acceptance.
   Recommended: A at ring20's next maintenance window; it is LAN-only and the most sensitive groups are already excluded,
   so it is lower urgency than B4.
4. **SMB mounts of Pen's documents without `seal`** (encryption in transit depends on the NAS setting; the inventory saw
   AES128-GCM negotiated). Recommended: add `seal` to the mount options after checking the NAS supports it (Pen's/dimitri-
   mint-dev's change, with the operator's go); low effort.
5. **/NasRing20 sshfs mounted with `allow_other`** (every local user on .107 can read the operator's admin NAS mount).
   Recommended: drop `allow_other` or restrict by group, after checking who needs it (1409 reads its corpus through it).
6. **Lower / informational:** dashboards over plain HTTP on the LAN (status only, no files: fan-dashboard, Watchtower);
   AEF's public GitHub repo (deliberate); review seats and the agent sessions sending content to model providers over
   HTTPS (inherent, known).

## Not in scope here
Cross-host file delivery design (O-1, step 4); VPN key rotation (T-3665).
