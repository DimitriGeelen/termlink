# Framework-agent prompt — fix `fw upgrade` infinite recursion

**Operator: copy everything inside the fenced block below into a fresh
`/opt/999-AEF` session (or wherever your framework-agent runs). The
prompt is self-contained — the agent does not need to see this header.**

---

```
URGENT — `fw upgrade` is unsafe to run on any consumer project with
`upstream_repo:` set in `.framework.yaml`. First real-world trigger
on 2026-06-06 at root@.107 in /opt/termlink burned ~16 GB disk + ~16 GB
GitHub bandwidth in 10 minutes before manual kill. Please prioritize.

## Symptom

Running `fw upgrade` in a consumer project spawns an infinite chain
of nested `fw upgrade <target>` invocations. Each iteration:
- Runs `git clone --depth=1` of the upstream repo (~191 MB) into a
  fresh `/tmp/...fw-upstream-XXXXXX/` tempdir
- Execs the just-cloned `bin/fw upgrade <target>`
- That child immediately re-detects bare-from-consumer and re-clones
- Loops at ~4 seconds per iteration with no natural termination

Banner repeats verbatim each iteration:

    Bare-from-consumer detected — auto-cloning upstream
      Upstream URL:  https://github.com/<owner>/agentic-engineering-framework.git
      Target:        /opt/<consumer>
      Cloning... ok
      Handing off to upstream's bin/fw: upgrade /opt/<consumer>

The trap-based tempdir cleanup at upgrade.sh:282 never fires because
the parent never returns — it's waiting on the child that's waiting
on its grandchild...

## Reproducer (file:// upstream — safe, no GitHub traffic)

    # 1. Set up a throwaway consumer with an upstream_repo pointing at
    #    a local clone of the framework repo:
    cd /tmp && rm -rf t-2014-{upstream,consumer}
    git clone --depth=1 /opt/999-AEF /tmp/t-2014-upstream
    mkdir /tmp/t-2014-consumer && cd /tmp/t-2014-consumer
    /opt/999-AEF/bin/fw init . --provider generic
    /opt/999-AEF/bin/fw vendor .
    cat >> .framework.yaml <<EOF
    upstream_repo: file:///tmp/t-2014-upstream
    EOF

    # 2. Trigger the loop (kill after 30s — do NOT let it run unbounded):
    cd /tmp/t-2014-consumer
    timeout 30s .agentic-framework/bin/fw upgrade 2>&1 | head -50

Expected (after fix): one `Bare-from-consumer detected` line, one
clone, hand off to upstream's bin/fw, complete upgrade, exit 0.

Observed (current bug): >5 `Bare-from-consumer detected` lines within
30s; `ps -ef | grep -c 'fw upgrade'` grows past 5.

Cleanup after repro: `rm -rf /tmp/t-2014-{upstream,consumer}` and kill
any survivors: `pkill -9 -f 'fw upgrade'`.

## Root cause (file:line)

`resolve_framework()` in `bin/fw:84-154` returns the wrong
FRAMEWORK_ROOT when `bin/fw` is invoked from an out-of-tree framework
checkout (the auto-clone tempdir is exactly such a checkout).

Step-by-step:

1. Consumer invokes `.agentic-framework/bin/fw upgrade`.
2. `resolve_framework` returns `<consumer>/.agentic-framework` (vendored).
3. `do_upgrade` in `lib/upgrade.sh:212-313` detects FRAMEWORK_ROOT
   equals the consumer's vendored copy at line 222 → enters the
   bare-from-consumer auto-clone branch.
4. Clones upstream to `$_tmpd/fw/`, then at line 310 execs:

       "$_tmpd/fw/bin/fw" upgrade "$target_dir"

5. **In the child process** (the upstream's bin/fw):
   - `FW_BIN_DIR=$_tmpd/fw/bin` → `candidate=$_tmpd/fw`
   - `candidate_is_framework_repo=1` (FRAMEWORK.md and `agents/`
     present in the freshly-cloned tree)
6. resolve_framework Step 1 (bin/fw:96-114) requires candidate to be
   AT or UNDER PROJECT_ROOT. `$_tmpd/fw` is in `/tmp`, target is in
   `/opt/<consumer>` → Step 1 fails.
7. **Step 2 (bin/fw:117-123) wins:**

       if [ -n "${PROJECT_ROOT:-}" ] && \
          [ -f "$PROJECT_ROOT/.agentic-framework/FRAMEWORK.md" ]; then
           echo "$PROJECT_ROOT/.agentic-framework"
           return 0
       fi

   This returns `<consumer>/.agentic-framework` — the same vendored
   copy that the parent already determined was bare-from-consumer.
   The explicit `$_tmpd/fw` handoff is **silently discarded**.
8. `do_upgrade` runs again; line 222 condition holds again; auto-
   clone fires again. Infinite loop.

The Step-2 "prefer vendored" rule (T-1346-B1 era) exists to handle
the global-shim case: `/usr/local/bin/fw` is just a router and is NOT
itself a framework repo (`candidate_is_framework_repo=0` there), so
vendored should win. The rule misfires here because the auto-clone
tempdir IS a real framework repo — explicitly handed off to — and
should be trusted.

## Why nothing caught this

- No depth guard / no recursion sentinel in upgrade.sh:212-313.
- The bare-from-consumer detection at upgrade.sh:222 is a state
  predicate (compare two paths). Re-entering the same state always
  re-fires; nothing in the protocol says "I am already a clone-of-a-
  clone, abort."
- The exec at upgrade.sh:310 does not set any sentinel env var that
  the child could honor.
- `tests/e2e/upgrade-test.sh` covers the happy path and the no-
  upstream-URL error path but does not exercise bare-from-consumer
  with a downstream `do_upgrade` exec.

## Recommended fix shape

### Primary — make resolve_framework respect explicit candidate

Two options; either is acceptable. Lean (1) for minimal blast
radius; (2) is more robust against unanticipated invocation paths.

(1) **Reorder rules in resolve_framework.** Move the "candidate IS a
framework repo" block (currently bin/fw:128-141) ABOVE the
vendored-preference block (bin/fw:117-123). Gate on candidate NOT
matching `*/Cellar/*` so the Cellar-with-vendored-consumer behavior
is preserved.

Sketch:

    # Step: if candidate is a real framework repo AND we were
    # invoked via it (not a global shim), trust the candidate.
    # Exception: Cellar paths inside a consumer project should still
    # fall through to vendored (preserves T-1346-B1 behavior).
    if [ "$candidate_is_framework_repo" = 1 ] \
       && [[ "$candidate" != */Cellar/* ]]; then
        echo "$candidate"
        return 0
    fi

    # (existing vendored-preference block stays here for the
    # global-shim / Cellar cases)

(2) **Sentinel env var.** In `lib/upgrade.sh:310`, set
`FW_FROM_AUTO_CLONE_HANDOFF=1` before exec. In
`bin/fw:resolve_framework`, before the Step-2 vendored block, honor
the marker:

    if [ -n "${FW_FROM_AUTO_CLONE_HANDOFF:-}" ] \
       && [ "$candidate_is_framework_repo" = 1 ]; then
        echo "$candidate"
        return 0
    fi

    # (existing Step-2 vendored preference)

The marker approach is also useful for the loop-detector below.

### Defense in depth — loop detector

In `lib/upgrade.sh:212-313`, increment a depth counter env var
across the exec at line 310:

    export FW_AUTO_CLONE_DEPTH=$(( ${FW_AUTO_CLONE_DEPTH:-0} + 1 ))

Then at the top of the bare-from-consumer branch (just after line
222 detects the condition), guard:

    if [ "${FW_AUTO_CLONE_DEPTH:-0}" -gt 0 ]; then
        echo "ERROR: auto-clone loop detected" >&2
        echo "  resolve_framework returned the consumer's vendored copy" >&2
        echo "  again (FRAMEWORK_ROOT=$FRAMEWORK_ROOT) despite being invoked" >&2
        echo "  from an upstream checkout. See T-2014." >&2
        echo "  Bin path: $0" >&2
        echo "  PROJECT_ROOT: ${PROJECT_ROOT:-(unset)}" >&2
        return 1
    fi

This is a backstop — even if the primary fix has a hole, the loop
bounds at depth 1.

### Regression test

Add to `tests/e2e/upgrade-test.sh`:

- Create a throwaway upstream (file://) + consumer pair (as in the
  reproducer above) inside the test sandbox.
- Run `fw upgrade` with a hard process-count guard:
  `( fw upgrade & ) ; sleep 5 ; test "$(pgrep -c -f 'fw upgrade')" -le 2`
- Cleanup tempdirs + processes in trap.

The test should be PURE file:// (no network). Fail if process count
exceeds 2 OR if `/tmp/fw-upstream-*` directories outnumber 1.

### Doc

Add a comment at `lib/upgrade.sh:310` (the exec site) stating the
invariant: "ANY exec of bin/fw outside the source-vendored tree
MUST propagate a sentinel that bypasses resolve_framework's Step-2
vendored fallback — else the child re-detects bare-from-consumer
and we loop. See T-2014."

## Acceptance — when is this done?

1. The reproducer above runs to completion in <30s with `fw upgrade`
   exiting 0, one clone in /tmp, one "Bare-from-consumer" banner.
2. `tests/e2e/upgrade-test.sh` includes the bare-from-consumer
   handoff regression and it passes.
3. `pgrep -c -f 'fw upgrade'` peaks at ≤ 2 during the test.
4. Upstream commit references T-2014; once mirrored, consumer-side
   T-2014 tracks the fix landing in its vendored `.agentic-framework/`
   via the next `fw upgrade` (which is the test itself).

## Consumer-side state (FYI for context)

- /opt/termlink T-2014 captures this RCA and the prompt artifact.
- /opt/termlink/.framework.yaml shows: `version: 1.6.160`, `upgraded_from: 1.6.260`, `upstream_repo: DimitriGeelen/agentic-engineering-framework`.
- T-1699 (the original framework-upgrade dispatch task) is blocked
  on this fix.
- No edits were made to /opt/termlink/.agentic-framework/ to fix the
  bug locally — fix MUST land upstream and propagate via the next
  (fixed) `fw upgrade`.

## Out of scope for this fix

- The 1.6.160 → latest upgrade itself (separate work, blocked on
  this).
- Any GitHub mirror behavior (G-058 unrelated).
- Watchtower UI changes (not impacted).
```

---

## Notes for the operator

- Paste **only** the fenced block above into the framework-agent session. The trailing prose ("Out of scope") is part of the prompt — the agent needs it to keep the fix tight.
- If your framework-agent runs in `/opt/999-AEF`, the agent already has the file paths to grep. No external context required.
- If `tests/e2e/upgrade-test.sh` already covers something similar, point the agent at the existing structure to amend rather than duplicate.
- When the fix lands and mirrors to GitHub, this consumer's `fw upgrade` should run to completion without the loop. At that point T-2014 closes via the verification block (`docs/reports/.../...framework-prompt.md` exists, RCA captured, post-fix `fw upgrade` clean).
