# T-3353 AC1: independent review of step 4 (architect) output

Reviewer: judge-t-3353-r1-c975edf4772f (rung-1-same-vendor-independent). Revision 3098980efeb0b4344896120fb05b3a96413fcc3a.
Verdict: **amber**

## What I checked

### Integrity of the hand-back
- `role-handback-step-04.json` output sha256 `935a3de4…715c` matches `docs/design/interactive-agent-communication-04-architecture.md` (ARC4) at this revision. The 04a sha256 `597988d7…af9a` given in the summary matches too.
- All six input hashes in the hand-back (REQ, TM, FP, RUL, ARCH2, LEASE) match the files at this revision (`sha256sum`).
- Card used: `docs/design/roles/cards/architect.md`, section 6 completion conditions 6.1 to 6.5. The chain YAML step 4 names this card and T-3353.

### 6.1 Every R-n, TH-n and SI-n addressed; no unexplained element: MET
- I wrote a script that pulls every `R-n` from REQ, `TH-n`/`BP-n` from TM and `SI-n`/`FX-n` from FP, then checks that each one appears in ARC4 or 04a. R-1..R-82: all 82 are rows in 04a table 2. TH-1..TH-65: all 65 are rows in table 3. SI-1..SI-31 and FX-1..FX-8 are in table 1.
- The only id missing is SI-32. It is a "consider an invariant [P]" inside FP CX-1, a change request to step 2, and is not in the TM floor. ARC4 takes up the substance of CX-1 as BP-20 [P] (section 14) and the hook-wiring control in 12.6, so this is not a gap.
- 04a table 4 maps each new element K-9a..K-31, the security levels, V-3 and GP-4 to its R/TH/SI. K-1..K-8 are built baseline from ARCH2. 4.1 raises the two elements without a clear source as XR-5 and XR-6.

### 6.2 Baseline, transitions and target, with boundaries and instrumentation: PARTLY MET
- Baseline: section 4 is a textual [B] baseline. Target: D-1 (5.2) shows trust boundaries TB-1/2/5/6/7/8/12 and [i] instrumentation points, with built and new parts coloured differently. D-2 (5.4) shows the network boundaries and what crosses them.
- Transitions: D-5 (11.1) and its text 11.2 list T1..T6 and the target, with what each step adds or removes.
- **Gap:** card 3.2 says "Each view shows trust boundaries and instrumentation points". D-5 and section 11 name no trust boundary (no TB-n) and no instrumentation point for any transition. Section 4 has no view that marks boundaries or [i]. It relies on the built colour in D-1. So these questions go unanswered:
  - which boundaries each transition creates or moves (for example TB-1/TB-2 becoming real at A3/T5; TB-7/TB-8 cross-hub cards at T4);
  - what instrumentation shows a transition took effect (for example status fields, PH-2 exit checks, consumer-list bind).
- The hand-back's evidence for 6.2 cites "D-1 with TB and [i] points" only for the target.

### 6.3 Every bypass route removed or a monitored residual: MET
- Section 14 has BP-1..BP-19, which is every BP in TM (19 ids found), plus BP-20 [P] and BP-21 [P]. Each row gives a design element and a removed, monitored or open-by-design outcome that cites an accepted RR-n.

### 6.4 Every open decision a decision record with options and a recommendation: MET
- Section 18 has DR-1..DR-14, each with context, options A/B(/C) and a recommendation. Operator questions O-1..O-7 are in section 21.
- The unruled step-3 OP-5/12/22/23 are surfaced as O-6, not assumed silently. V-16 (GP-11) is explicitly left open.
- Minor: card 3.8 asks for "decision" and "status" fields. The table has no such columns; "all proposed" is stated only in the section heading.

### 6.5 Drawings D-1..D-5 with id, caption, text equivalent, and render without error: MET
- Captions: 5.2 D-1, 5.4 D-2, 7.6 D-3, 10.1 D-4, 11.1 D-5. Text equivalents: 5.3, 5.5, 7.7, 10.2, 11.2.
- D-3 has a primary flow and a refusal flow; D-4 has three state machines.
- I re-rendered all five Mermaid blocks with `mmdc` and `/usr/bin/chromium --no-sandbox`. Every block exited 0. The SVG sizes were 47324, 26237, 43435, 73445 and 24588 bytes, identical to the figures in ARC4 24.2. The SVGs contain no "syntax error" or "parse error"; the only "error" strings are CSS class names.

### Spot-checks of [B] baseline claims against the code
- `termlink-hub/src/tls.rs` ~line 47: `hub_id_from_fingerprint`, first 16 hex. Its comment defers minting to step 4. Matches 4.6.
- `termlink-session/src/handler.rs`: `handle_command_notify` is at line 618 (the document cites 611, close enough). The test asserts the line `[sidecar] N peer messages waiting (ids …)`, so the brackets and parentheses claim behind DR-6 is true. The TM SI-1 alphabet is letters, digits, spaces and `. , : -`.
- `termlink-session/src/auth.rs`: the `notify` scope sits below `interact` (T-3533). Matches 4.4.
- `crates/termlink-sidecar/src/` contains `lease.rs`, `liveness.rs` and `activity.rs`, as cited.

### Fit as input for step 5 (evidence specialist)
- Mostly fit. Section 6 gives a proof and a mutant for every PH-2 row. Section 15 gives each component a production property, how it is checked, and what the check does NOT prove. Section 22 gives build items, and every PH-2 floor item has a named probe owner (04a table 1).
- Gap for step 5: section 15 (card 3.7, "per component") leaves out K-12 ladders, K-17 injectors, K-24 encryption keys and K-25 the mail-hub declaration. Those are PH-2 components the evidence specialist must specify evidence for.
- Gap for step 5: FP 14.1.b items such as "the claim mechanism and the transcript-complete signal per harness (R-51.a, R-47.a)" and "composition of the BV" get no explicit disposition. Both are arguably covered as built baseline (SI-30, PH-1 X1..X6), but ARC4 does not say so.

## Needed for green
1. Section 11 (and the D-5 text, or a table beside it): per transition T1..T6/target, name the trust boundaries introduced or changed (TB-n) and the instrumentation points that show the transition took effect. Either add a baseline view marked with TB/[i], or state explicitly that D-1's built-coloured subset is the baseline view.
2. (Minor) Section 15: add rows for K-12, K-17, K-24 and K-25 (PH-2). Optionally add a line for the PH-3+ components saying their measurement is deferred to their sub-designs.
3. (Minor) Section 18: add a status/decision column or note per DR (card 3.8). Add a line disposing of each FP 14.1.b hand-over item, including those satisfied by the build.
