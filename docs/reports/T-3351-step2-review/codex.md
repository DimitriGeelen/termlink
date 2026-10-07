## 1. Completion conditions: not yet met

**1a. 6.1 — Structurally complete, substantively incomplete.** Section 14 supplies all ten questions for thirteen boundaries, but several answers rely on controls that do not establish the claimed protection. Examples: approval replay is answered by expiry alone (§15.10.6); transcript forgery by attacker-writable record types (§13.2); credential replay by unspecified channel binding (§7.2). These need correction before declaring coverage complete.

**1b. 6.2 — Traceability met; adequate disposition not met.** Every TH has a countermeasure or residual reference, but several countermeasures overclaim and RR-7 understates undetectable compromise. Some threats also lack the role card’s required likelihood/impact reasons: TH-12, TH-20 and TH-21 are examples.

**1c. 6.3 — Broad inventory, completeness unproven.** Section 17 covers all route categories explicitly named in the supplied role card. However, “monitored” is misleading where the adversary can also alter the monitor, its credentials and evidence. The unavailable profile cannot independently establish completeness.

**1d. 6.4 — Not met.** SI-17 cannot reject a correctly formatted forged harness record merely through nonce and record-type checks. SI-22 cannot detect every deletion through a hash chain alone. SI-21’s unwritable policy storage is unsupported on the stated root-sharing deployment.

**1e. 6.5 — Partially evidenced.** All drawings have identifiers, captions and textual equivalents; §24 reports successful rendering, which this text-only review cannot verify. D-1 omits the receiver-store-to-hook flow and draws TB-6 “same host” between different host zones. D-2 routes the receiver’s reply to the sender’s sidecar, bypassing its own sidecar.

## 2. Coverage and technical defects

**2a. Circuit lifetime is not securely bounded.** Section 7.2.d starts expiry at first acceptance, without bounding the delay between issuance and acceptance. A previously unused credential presented long after revocation can receive a fresh lifetime. Channel binding does not establish issuance freshness. Require authenticated fresh establishment, a non-resettable expiry basis, and tests for delayed presentation, renewal, reconnect and restart. RR-5’s one-hour bound is currently unsupported.

**2b. Revocation needs separate scopes.** Sections 11 and SI-11 conflate role revocation, identity revocation and conversation authority. Allow-list changes can remain stale throughout an outage, while SI-23 promises fleet-wide effect within one tick. State reachable and partitioned bounds separately. Define revocation-list rollback protection and what happens when revocation retrieval fails while other hub operations work.

**2c. Fleet admission leaves authority bootstrapping unresolved.** PR-5 authenticates a hub, but first registration establishes project ownership without clearly proving entitlement to that project id. Model project-id squatting, malicious competing claims, stale roster restoration, hub-key rotation/recovery and trust transfer during re-home. An enrolled rogue hub can cause denial by claiming another project even when nobody believes the claim.

**2d. Ordering and identity need stronger definitions.** PR-4 must acknowledge the highest **contiguous durably stored** number, not merely a number no higher than the maximum stored. Otherwise receipt of 1 and 3 can conceal missing 2. PR-2/PR-3 also need continuity across signing-key rotation, or rotation creates fresh deduplication and sequence namespaces.

**2e. Important paths remain under-modelled.** Add many-to-many membership, per-recipient receipts and confidentiality; explicit conversation hand-over under R-62; readiness-to-injection races; replay of one-off approvals within their validity period; compromised approval-device/session capabilities; and crash recovery between actual hand-over and recording HANDED_OVER. A nonce does not resolve that last duplicate-delivery ambiguity.

## 3. Fidelity to approved requirements

**3a. Interrupt permission becomes delivery permission.** Sections 7.2.a, 7.6.a and SI-10 use the interrupt allow-list as a circuit admission gate. Step 1 R-52.a and R-63.a require disallowed urgency to be downgraded, never dropped. Separate ordinary communication admission from urgent permission; no CR currently authorizes this restriction.

**3b. Revocation becomes death.** Section 11.1 dead-letters mail as DEAD when a key is revoked. Step 1 R-44.a/e and R-60.a reserve DEAD for the home hub’s determination that an instance ended. Revocation does not establish termination. Raise a CR or preserve that distinction.

**3c. Role takeover is narrowed.** PR-15/TH-54 require no process to hold the session before a **takeover**. Step 1 R-67 permits takeover after lease lapse and R-68 requires takeover from a stuck holder with a live sidecar. CR-12 mentions this change but does not disclose the conflict with R-67/R-68. Separate duplicate-session resume prevention from role succession.

**3d. Retention is misstated.** RR-10 says telemetry and stage memory expire after fourteen days. Step 1 R-35.o specifies fourteen days **after final state**, preserves open messages subject to explicit ceiling exceptions, and retains digests for one year. Correct the risk rather than implying routine expiration of open-message memory.

**3e. Other changes need explicit impact mapping.** Section 8.3 fixes participants at creation without accommodating R-62’s explicit hand-over. PR-5’s “both disbelieved” project-wide response exceeds R-60’s second-instance treatment. CR-1 addresses hub-first writes but must also cover R-26’s reply retrieval and R-44’s UNKNOWN semantics during outages.

## 4. Residual risks

**4a. RR-1 and RR-3** identify real model-behaviour risk. Retain them, but distinguish enforced permission boundaries from instructions the model may ignore. Caps do not bound the harm of one malicious message.

**4b. RR-2 and RR-7** require substantial correction. Root can modify binaries, monitors, policy files and all local evidence, not merely steal keys. A forged reply can satisfy the reply deadline; selective honest canary handling can conceal targeted loss indefinitely. Neither detection nor its deadline is guaranteed. Separate users alone do not contain host root, and unspecified containers are not sufficient.

**4c. RR-4, RR-5 and RR-14** are legitimate categories but depend on unresolved bootstrap, lifetime and approval-channel protections. A compromised home hub may substitute the public key used for verification unless an independent binding prevents it; “cannot forge an agent’s signature” understates effective impersonation.

**4d. RR-6** is a genuine confidentiality tradeoff, but “accepted by default today” in TH-22 is unauthorized. Separate exposure to the hub operator from exposure to ordinary token holders; read scopes mitigate the latter.

**4e. RR-8** incorrectly groups induced lapse with duplicate-authority conflict: normal lapse should permit automatic takeover. **RR-9** is honest, subject to trusted display and execution binding. **RR-10** needs the retention correction above. **RR-11** is legitimate, but host availability does not imply hub-process availability. **RR-12** needs aggregate resource reservation and fairness analysis. **RR-13** is candid; preserving injection does not require preserving every current authorization weakness.

**4f. Missing residuals** include undetectable log truncation/forking without independent checkpoints, compromised approval devices, and ambiguous delivery recovery.

## 5. Decision rights and recommendations

**5a.** OQ-1’s “network attacker already covered by TLS” option is misleading: TLS does not prevent delay or denial. OQ-6 falsely reduces interim operator authentication to a new key or unverifiable claims; existing authenticated channels deserve assessment.

**5b.** OQ-3’s outage-duration claims and PN-5/PN-7 sizing lack supplied measurements. Label them hypotheses. OQ-5 mixes independent durability choices; OQ-7 mixes token-reader isolation with hub confidentiality.

**5c.** Remove “RR-2 already accepts” (§7.3.a). Distinguish proposed invariants dependent on CR approval from existing obligations; the blanket DRAFT disclaimer does not resolve contradictory “firm invariant” language.

## 6. Handoff verdict

**6a. Return for revision.** The structure and traceability are useful, but the next role would inherit impossible guarantees and hidden requirement changes. Correct those first, then provide an invariant-to-CR dependency table identifying pending operator decisions, adversary limits and achievable probes.
