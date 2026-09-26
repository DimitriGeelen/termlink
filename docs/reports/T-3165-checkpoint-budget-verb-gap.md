# T-3165 — `checkpoint.sh budget`: a verb three documents prescribe, one allowlist blesses, and no code implements

**Status:** measured 2026-09-26. Origin: the `/resume` skill failed on its own step 6 during an
ordinary session start.

## What happened

`/resume` step 6 says to read the context budget with
`.agentic-framework/agents/context/checkpoint.sh budget`. Run as written, it produces:

```
Usage: checkpoint.sh {post-tool|reset|status}

╔══════════════════════════════════════════════════╗
║  HOOK CRASHED: checkpoint (exit 1)              ║
║  This is a hook malfunction, NOT a policy block ║
║  Action: Report to human, run fw doctor         ║
╚══════════════════════════════════════════════════╝
```

The failure direction is the point. This is not a quiet "unknown verb" — the banner tells the
agent it is looking at **a hook malfunction** and instructs it to report to the human and run
`fw doctor`. So following the skill as written sends the reader chasing a non-existent
infrastructure fault at the very start of every session, before any work begins. The fault is
in the instruction, and the diagnostic says to go look at the machine.

## Measured

`checkpoint.sh` dispatches exactly three cases — no `budget`:

| line | case |
|---|---|
| 277 | `post-tool)` |
| 427 | `reset)` |
| 439 | `status)` |
| 458 | `Usage: checkpoint.sh {post-tool\|reset\|status}` |

Three on-disk copies of the `/resume` skill give **three different answers** to the same step:

| copy | provenance | prescribes | works? |
|---|---|---|---|
| `~/.claude/commands/resume.md` | user settings — **the copy actually loaded** | `checkpoint.sh budget` | **no** — exit 1 + HOOK CRASHED |
| `.claude/commands/resume.md` | project, git-tracked | `cat .context/working/.budget-status` | yes, but see below |
| `.agentic-framework/lib/templates/resume-md.md` | framework template (vendored) | neither — no budget step | n/a |

And a fourth answer, the correct one, is in `CLAUDE.md` itself (the T-3127 note): under
concurrent multi-worker dispatch use **`checkpoint.sh status`**, because `.budget-status` is a
single path shared by every dispatched worker and holds whichever wrote it last.

So the project copy prescribes precisely the raw read that CLAUDE.md documents as unreliable,
and the loaded copy prescribes a verb that does not exist. Neither is the answer CLAUDE.md
gives. Same class as T-2484 (the charter sentence as three copies with no transclusion) applied
to the session-start budget read: markdown has no transclusion, so the copies forked and nothing
compared them.

## The sharp finding — an allowlist entry for code nobody wrote

`agents/context/lib/safe-commands.sh:685` (vendored):

```bash
checkpoint.sh)
    local cp_sub
    cp_sub=$(echo "$cmd" | awk '{print $2}')
    case "$cp_sub" in
        status|budget)
            return 0
            ;;
    esac
    ;;
```

`budget` is granted unconditional read-only passage through the P-002 write gate. The comment
immediately above that arm audits the verb surface **verb by verb** and enumerates exactly
three:

> `status` was read to verify: its only write is the ensure_counter bootstrap […] The mutating
> arms (`post-tool` increments counters and can trigger auto-handover; `reset` deletes session
> state) fall through and stay gated

It never mentions `budget`. So `budget` is in the allowlist **unjustified by the reasoning
written beside it** — the one place in the file that documents why each verb is or is not safe.

The risk is not the missing verb; it is the pre-granted exemption. Whoever implements
`checkpoint.sh budget` next inherits a gate that already calls it read-only, without review. If
that implementation writes anything — a repaired cache, a marker file, a stale-cache eviction,
all plausible for a verb whose job is rejecting bad caches — it passes P-002 silently on day
one. An allowlist entry for unwritten code is a review that already happened, for code that
does not exist yet.

That the allowlist knows the name is also the evidence `budget` was **designed** upstream and
the implementation did not land. This is a half-shipped verb, not a typo in a skill.

## Attribution

- **Vendored (upstream, G-062 — filed, not patched):** the `safe-commands.sh:685` allowlist entry
  and the absent `checkpoint.sh budget` implementation. Both live under `.agentic-framework/`.
  Upstream should either implement the verb (and extend the audit comment to justify it) or drop
  it from the allowlist. Dropping it is the safer default: the gate should not bless a verb that
  cannot be read.
- **Ours (fixed here):** `.claude/commands/resume.md`, git-tracked, prescribing the raw
  `.budget-status` read that the T-3127 CLAUDE.md note documents as unreliable under concurrent
  dispatch. Repointed at `checkpoint.sh status`, the verb that exists and is per-process.
- **The operator's (reported, NOT edited):** `~/.claude/commands/resume.md` is outside the
  project boundary (T-559). It is the copy that actually loads, so it is the copy that matters
  most, and it is the one I must not silently change. It needs the same one-word edit:
  `budget` → `status`.

## Why nothing caught it

The `/resume` skill is prose, and no guard reads it. The three static checks that would be the
natural home each ask a different question: T-2561 asks whether a declared crontab is installed,
T-2814/T-2817 ask whether framework files are tracked and whether their `source`/`exec`
references resolve. **None asks whether a documented command's sub-verb exists.** The
`safe-commands.sh` allowlist is the one artifact in the tree that already enumerates the verb
surface, which is exactly why it was able to name a verb the script lacks and have nobody notice.

A check for this class would compare sub-verbs named in operator-facing docs and in the
safe-commands allowlist against the `case` arms the target script actually dispatches. That is a
real, bounded check, and it is not written. Logged as the follow-up rather than built here —
this task's deliverable is the measurement and the filing.
