# T-180: Dual-Path Upstream Reporting — Design Research

## Problem Statement

When a consumer project (like TermLink) discovers improvements needed in the framework, there's no standard way to report them upstream. Currently the process is ad-hoc: manually crafting prompts and injecting them via TermLink TOFU (proven in T-184/T-185) or creating issues/notes manually.

We need a documented, repeatable process with two paths:
1. **PRIMARY (TermLink):** Direct injection into framework agent session — immediate, interactive
2. **FALLBACK (fw upstream):** File-based report when TermLink is not available — async, portable

## Path 1: TermLink inject-remote (PRIMARY)

### When to use
- Framework agent is running on another machine (or locally in another terminal)
- TermLink hub is running with TCP on the remote machine
- You have the hub secret
- You want immediate, interactive feedback

### How it works (post T-187)
```bash
# One-liner: inject improvement prompt into framework agent session
termlink remote inject <hub-host:port> <session-name> \
  "FRAMEWORK IMPROVEMENT: <title>

  Context: <what was discovered, which task, what evidence>
  Artifacts: <file paths, code references>
  Suggestion: <proposed improvement>

  APPLY FRAMEWORK GOVERNANCE: Create an inception task for this.
  Ask me if you need more information." \
  --secret-file ~/.termlink/hub.secret --enter
```

### Prerequisites
1. Remote hub running: `ssh <host> 'termlink hub start --tcp 0.0.0.0:9100'`
2. Framework agent session registered: `ssh <host> 'termlink register --name fw-agent --shell'`
3. Hub secret available locally: `scp <host>:~/.termlink/hub.secret ~/.termlink/hub.secret`
4. TOFU fingerprint accepted (automatic on first connect)

### Prompt template for improvements
Each improvement prompt should include:
- **Title:** Clear one-line description
- **Context:** Task ID, what was being done when discovered
- **Evidence:** Specific files, line numbers, error messages, patterns observed
- **Artifact references:** `docs/reports/T-XXX-*.md`, episodic summaries, learnings
- **Governance instruction:** "APPLY FRAMEWORK GOVERNANCE: Create inception task. Ask me if you need more info."

### Proven track record
- T-184: First cross-machine injection (681 bytes)
- T-185: 5 improvement prompts (7.4KB total), all successfully injected
- Topics: episodic enrichment, inception commit gate, human AC format, context budget, audit RCA

## Path 2: fw upstream report (FALLBACK)

### When to use
- No TermLink hub available (remote machine offline, no network, air-gapped)
- Want to batch multiple findings into a structured report
- Need a persistent file artifact that can be committed and reviewed later

### Proposed command (framework enhancement)
```bash
# Create an upstream improvement report
fw upstream report \
  --title "Episodic enrichment should be mandatory" \
  --context "Discovered in T-183: auto-generated episodics miss challenges and successes" \
  --evidence "See .context/episodic/T-183.yaml before enrichment" \
  --suggestion "Add enrichment_status: pending as default, block completion until enriched" \
  --attach-doctor \
  --task T-183
```

### Output
Creates a YAML file in `.context/upstream/` (or `docs/upstream/`):
```yaml
---
id: U-001
title: "Episodic enrichment should be mandatory"
created: 2026-03-19T06:00:00Z
source_project: 010-termlink
source_task: T-183
status: pending  # pending -> sent -> acknowledged -> implemented -> rejected
context: "..."
evidence: "..."
suggestion: "..."
doctor_output: "..."  # if --attach-doctor
---
```

### Delivery
The upstream report file can be:
1. Injected via TermLink when hub becomes available
2. Committed to the framework repo as a PR/issue
3. Emailed or shared as a file
4. Read by `fw harvest` during framework upgrade cycles

## Comparison

| Aspect | TermLink (Primary) | fw upstream (Fallback) |
|--------|-------------------|----------------------|
| Latency | Immediate | Async (file-based) |
| Interactivity | Yes (agent can ask follow-ups) | No (one-shot) |
| Requires network | Yes (TCP to hub) | No |
| Requires TermLink | Yes | No (framework only) |
| Governance | Agent creates inception task | Human reviews file later |
| Batch support | One prompt at a time | Can batch multiple |
| Persistence | Injected into session (ephemeral) | File on disk (permanent) |

## Recommendation

1. **Document both paths** in framework CLAUDE.md or a dedicated guide
2. **TermLink path is ready now** (T-187 implemented `termlink remote inject`)
3. **fw upstream command** needs framework changes — send as improvement prompt via TermLink to framework agent
4. **Template the prompt format** so improvements are structured consistently

## Implementation Plan

- [x] TermLink inject-remote command (T-187 — DONE)
- [ ] Document TermLink upstream workflow in project docs
- [ ] Send `fw upstream report` proposal to framework agent via TermLink
- [ ] Framework agent creates inception task for `fw upstream` command
