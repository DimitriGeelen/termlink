# T-534: RCA — Agent Repeatedly Suggests Direct GitHub Push

## Symptom

Agent keeps suggesting `git push github main --tags` despite:
1. Memory explicitly saying "only push to onedev"
2. User correcting this multiple times

## Root Causes

### RC-1: Memory exists but lacks structural enforcement
The feedback memory says "don't push to GitHub" but there's nothing preventing the agent from suggesting it. Memory is guidance, not a gate.

### RC-2: Release workflow creates false dependency
`.github/workflows/release.yml` triggers on GitHub tags. The agent sees this and concludes "tags must be pushed to GitHub manually." **Wrong** — OneDev auto-mirrors.

### RC-3: OneDev→GitHub mirror not documented in CLAUDE.md
The `.onedev-buildspec.yml` has a `PushRepository` job that auto-mirrors every branch/tag to GitHub. But CLAUDE.md never mentions this, so the agent doesn't know it exists.

### RC-4: Homebrew formula hardcodes GitHub URLs
`homebrew/Formula/termlink.rb` references `github.com/.../releases/download/...`. Agent sees this and thinks GitHub is the primary distribution channel requiring manual action.

## The Actual Flow

```
git push origin main --tags  (OneDev)
        ↓
OneDev buildspec: PushRepository → GitHub (automatic)
        ↓
GitHub Actions: release.yml triggers on v* tag (automatic)
        ↓
GitHub Releases: binaries + checksums published (automatic)
        ↓
Homebrew: brew install works (automatic)
```

**Everything after `git push origin` is automated.** Zero manual GitHub interaction needed.

## Evidence

- `.onedev-buildspec.yml`: `PushRepository` step with `github-push-token` secret, triggers on `BranchUpdateTrigger` + `TagCreateTrigger`
- Tags v0.1.0 and v0.9.0 present on both OneDev and GitHub (mirror works)
- v0.1.1 missing on GitHub (possible mirror hiccup, not blocking)

## Mitigations

### M-1: Update memory with WHY (done)
Strengthen the feedback memory to explain the auto-mirror chain, not just "don't push to GitHub."

### M-2: Document mirror in CLAUDE.md (recommended)
Add a section to CLAUDE.md explaining the push→mirror→release chain. This survives across sessions.

### M-3: Remove `github` remote from local config (optional)
If the `github` remote doesn't exist, the agent can't suggest pushing to it. Downside: breaks manual GitHub operations if ever needed.

## Recommendation

**GO on M-1 + M-2.** Document the mirror chain in both memory and CLAUDE.md so every session understands it structurally.
