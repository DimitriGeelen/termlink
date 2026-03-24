# Framework Fix F3: Add Architectural Context Section to Handover Template

## Pickup Prompt for Framework Agent

### Problem

Handovers are narrative prose — they describe "what happened" and "what to do next" but not "what architectural principles govern the work." A new session reads the handover, sees task names, but has no structured access to the decisions driving those tasks.

### Real-world impact (T-258 incident)

The handover said "T-239, T-240, T-241, T-242 — captured, needs inception" with no context about how they form a layered architecture. A new session evaluated each as an isolated feature.

### Files to modify

**`agents/handover/handover.sh`**
- Lines 313-497: Main handover template
- Insert new section between "Work in Progress" (~line 331) and "Decisions Made This Session" (~line 448)

### Proposed fix

Add a new `## Architectural Context` section to the handover template that auto-populates from `decisions.yaml`:

```bash
## Architectural Context

<!-- Auto-populated from .context/project/decisions.yaml (project-scoped entries) -->
```

The handover generator should:
1. Read `.context/project/decisions.yaml`
2. Filter to `scope: project` entries
3. Group by task ID
4. Render as a compact reference list:

```
## Architectural Context

Active architectural decisions (from decisions.yaml):
- **D-004** (T-233): GO on specialist agent orchestration via TermLink
- **D-005** (T-233): Deterministic-first execution with stochastic fallback
- **D-006** (T-233): Framework owns policy, TermLink owns mechanism
- **D-007** (T-233): Layered capability discovery with progressive autonomy
- **D-008** (T-233): Qualitative trust supervision — not run counters
```

**Design principles:**
- Compact — one line per decision, not full rationale (that's in decisions.yaml)
- Auto-populated — no human/agent effort to maintain
- Queryable — next session can grep for decision IDs
- Only project-scoped decisions (not framework universals)

### Acceptance criteria

- [ ] Handover template includes `## Architectural Context` section
- [ ] Section auto-populates from project-scoped entries in `decisions.yaml`
- [ ] Empty `decisions.yaml` produces "No project decisions recorded" (not a blank section)
- [ ] Format is one line per decision with ID, task ref, and decision text

### Test commands

```bash
# Generate a handover
fw handover
# Verify architectural context section exists
grep -q "Architectural Context" .context/handovers/LATEST.md && echo "PASS" || echo "FAIL"
# Verify decisions are listed
grep -q "D-004\|D-005" .context/handovers/LATEST.md && echo "PASS (has decisions)" || echo "INFO (no decisions yet)"
```
