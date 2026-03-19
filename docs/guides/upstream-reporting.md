# Upstream Reporting: Consumer-to-Framework Feedback

When working in a consumer project (e.g., TermLink) and discovering framework improvements, use one of two paths to report upstream.

## Path 1: TermLink inject-remote (Primary)

Direct injection into a framework agent's Claude Code session on another machine. Immediate, interactive — the framework agent can ask follow-up questions.

### Prerequisites

1. **Remote hub running** with TCP listener:
   ```bash
   # On the framework machine:
   termlink hub start --tcp 0.0.0.0:9100
   ```

2. **Framework agent session registered**:
   ```bash
   # On the framework machine:
   termlink register --name fw-agent --shell
   ```

3. **Hub secret available locally**:
   ```bash
   # Copy secret from remote to local machine:
   scp user@framework-host:/tmp/termlink-0/hub.secret ~/.termlink/hub.secret
   ```

4. **TOFU fingerprint** — accepted automatically on first connect. If the hub cert changes (hub restart), clear stale entries:
   ```bash
   rm ~/.termlink/known_hubs
   ```

### Usage

```bash
termlink remote inject <host:port> <session> "message" \
  --secret-file ~/.termlink/hub.secret --enter
```

### Example: Send an improvement finding

```bash
termlink remote inject 192.168.10.107:9100 fw-agent \
  "FRAMEWORK IMPROVEMENT: Episodic enrichment should be mandatory

Context: Discovered in TermLink T-183. Auto-generated episodic summaries
miss challenges and successes — the most valuable parts of episodic memory.
After manual enrichment, T-183 episodic went from generic to genuinely useful.

Evidence:
- Before: 'challenges: # No challenges detected in commit messages'
- After: 3 specific challenges (hub.auth method, TOFU stale fingerprint, hub routing)
- File: .context/episodic/T-183.yaml

Suggestion: Add enrichment_status: pending as default. Block task completion
until episodic is enriched (or explicitly marked as mechanical).

APPLY FRAMEWORK GOVERNANCE: Create an inception task for this.
Ask me if you need more information." \
  --secret-file ~/.termlink/hub.secret --enter
```

### Prompt Template

Every improvement prompt should follow this structure:

```
FRAMEWORK IMPROVEMENT: <one-line title>

Context: <task ID, what you were doing when discovered, why it matters>

Evidence: <specific files, line numbers, before/after, error messages>

Suggestion: <proposed improvement with enough detail to act on>

APPLY FRAMEWORK GOVERNANCE: Create an inception task for this.
Ask me if you need more information.
```

### Options

| Option | Description |
|--------|-------------|
| `--secret-file <path>` | Path to hex secret file (recommended) |
| `--secret <hex>` | Hex secret inline (for scripting) |
| `--enter` | Append Enter keystroke (submits the prompt) |
| `--delay-ms <ms>` | Inter-key delay [default: 10] |
| `--scope <scope>` | Auth scope [default: control] |
| `--json` | Output result as JSON |

### Troubleshooting

| Error | Fix |
|-------|-----|
| "Cannot connect" | Check hub is running with `--tcp`, port is open |
| "Authentication failed" | Verify secret matches remote hub.secret |
| "TOFU VIOLATION" | Hub cert changed (restart). Run `rm ~/.termlink/known_hubs` |
| "Session not found" | Check session name with `ssh host 'termlink list'` |

---

## Path 2: fw upstream report (Fallback)

File-based report when TermLink is not available (offline, air-gapped, no hub). Creates a structured YAML file that can be delivered later.

> **Note:** This command does not exist yet in the framework. It is proposed below as a framework enhancement. Until implemented, manually create the report file.

### Proposed Command

```bash
fw upstream report \
  --title "Episodic enrichment should be mandatory" \
  --context "Discovered in T-183: auto-generated episodics miss challenges" \
  --evidence "See .context/episodic/T-183.yaml before enrichment" \
  --suggestion "Add enrichment_status: pending, block completion until enriched" \
  --attach-doctor \
  --task T-183
```

### Output Format

Creates a YAML file in `.context/upstream/`:

```yaml
---
id: U-001
title: "Episodic enrichment should be mandatory"
created: 2026-03-19T06:00:00Z
source_project: 010-termlink
source_task: T-183
status: pending
context: "Discovered in T-183..."
evidence: "See .context/episodic/T-183.yaml..."
suggestion: "Add enrichment_status: pending..."
doctor_output: "..."
---
```

### Manual Workaround (Until fw upstream is implemented)

Create the file manually:

```bash
mkdir -p .context/upstream
cat > .context/upstream/U-001-episodic-enrichment.yaml << 'EOF'
---
id: U-001
title: "Episodic enrichment should be mandatory"
created: 2026-03-19T06:00:00Z
source_project: 010-termlink
source_task: T-183
status: pending
context: |
  Discovered in T-183: auto-generated episodic summaries miss challenges
  and successes — the most valuable parts of episodic memory.
evidence: |
  Before enrichment: challenges section was empty placeholder
  After enrichment: 3 specific challenges documented
suggestion: |
  Add enrichment_status: pending as default.
  Block task completion until episodic is enriched.
---
EOF
```

### Delivery Options

Once the report file exists, deliver it via:

1. **TermLink** (when hub becomes available): inject the file content via `termlink remote inject`
2. **Git commit**: commit to the framework repo as PR/issue
3. **fw harvest**: framework's `fw harvest` picks up upstream reports during upgrade cycles
4. **Manual**: email, Slack, or shared drive

---

## When to Use Which Path

| Scenario | Path |
|----------|------|
| Framework agent running on another machine | **TermLink** (Path 1) |
| Framework agent running locally in another terminal | **TermLink** (Path 1, use localhost) |
| No network / offline | **fw upstream** (Path 2) |
| Batch multiple findings | **fw upstream** (Path 2) |
| Need interactive follow-up | **TermLink** (Path 1) |
| Want persistent record | **fw upstream** (Path 2), then optionally inject via TermLink |

## History

- T-184: First cross-machine injection proven (681 bytes)
- T-185: 5 improvement prompts (7.4KB) sent to framework agent
- T-186: Inception — designed `termlink remote inject` CLI command
- T-187: Implemented `termlink remote inject`
- T-180: Inception — designed dual-path upstream reporting
