# T-2022 Inception Research — Substrate primitive #4: filesystem-write observation (CSMA/CD collision detection)

**Status:** DEFER as captured. Kernel-level FS observation has unsolvable host-portability + capability constraints for the ring20 deployment. Re-scope as a sub-arc exploring git-hook-enforced path-touched declaration instead.
**Artifact created:** 2026-06-08
**revisit_at:** 2026-09-08 (90 days — pair with T-2024's revisit; lets Foundation primitives ship and AEF accumulate evidence on whether conservative policy is acceptable in practice)
**revisit_evidence_needed:** Either (a) a concrete AEF-layer incident attributable to lacking write-observation; (b) a successful spike of git-hook-enforced path declaration; or (c) a ring20 deployment-shape change that opens up CAP_BPF or CAP_SYS_ADMIN.
**See also:** T-2018 ADR §4 (collision policy boundary), §6 #4; this is the KEYSTONE primitive that gates the conservative→optimistic flip.

## 1. The §6 framing

ADR §6 primitive #4 + §4: *"An exhaustive code search found no inotify / fanotify / fs-watch / equivalent at runtime. The hub cannot see what files an agent touches. This absence FORCES the conservative collision policy in §4. Its presence is the *precondition* (not the trigger) for an optimistic mode."*

ADR §4 explicitly states: *"Building write-observation does NOT flip the policy. It creates the option to go optimistic later; the flip is a separate, gated decision."* The inception here authorizes only the option, not the flip.

## 2. Mechanism survey

Six candidate mechanisms, scored against ring20's deployment shape:

| Mechanism      | Portability        | Capability needed     | Blind spots                                          | Perf cost | Verdict for ring20 |
|----------------|--------------------|-----------------------|------------------------------------------------------|-----------|-------------------|
| inotify        | Linux only         | None (user)           | Per-directory add_watch; queue-overflow silent drop; cross-mount blind | Low (~3-5%) | ❌ Linux-only blocks macOS-in-fleet |
| fanotify       | Linux only         | CAP_SYS_ADMIN         | Cross-mount via mount-mark; can lose events on heavy load | Low-moderate | ❌ Container CAP-drop common; Linux-only |
| ptrace         | Linux+BSD         | None for own process  | ~10× syscall slowdown; per-PID attach; misses async I/O | Severe | ❌ Perf cost makes agents unusable |
| LD_PRELOAD     | POSIX libc        | None                  | Statically-linked binaries bypass; direct syscall bypasses; Rust binaries variable | Low | ❌ Self-observation gap (Rust agents) |
| eBPF           | Linux ≥4.18       | CAP_BPF or root       | Tooling complexity; kernel-version compat; container restrictions | Very low | ❌ Container caps usually dropped |
| FUSE           | POSIX             | CAP_SYS_ADMIN to mount | Agents must write through FUSE mount; breaks normal POSIX semantics | Severe | ❌ Workflow incompatible |

**Cross-mechanism conclusion:** No mechanism cleanly satisfies (portable across Linux + macOS) AND (works without elevated container capabilities) AND (catches Rust agent self-writes) AND (acceptable perf cost). The intersection is empty.

## 3. The deeper question — is OS-level FS observation the right abstraction?

The §6 framing assumes kernel-level FS-watching is the natural primitive. But step back:

- **What does the orchestrator actually need to detect?** Specifically: two agents working in parallel both touching the same file in the same code plane.
- **Where do AEF agents touch files?** Inside their assigned git worktree (per ADR §5 plane split: code plane wants isolation).
- **When does a collision become a real problem?** At commit/merge time, not at the moment of write.
- **What is the "honor-system" critique actually about?** §4 says optimistic-on-honor-system is unsafe because "an agent that forgets to announce" produces undetected collisions. The fix isn't kernel observation; it's making the announcement *structural*, not voluntary.

If announcement happens via a **pre-commit git hook** that posts `{agent_id, paths: [...]}` to a topic before any commit lands, the agent can't bypass it without disabling the hook — and the hook-disable itself is observable (the next operation that DOESN'T post is the smoking gun).

This is honor-system in *form* (the hook is software) but enforced in *practice* (the workflow path is the only path; bypassing means deliberately breaking the deployment). For cooperating AEF agents in a single trust domain, that's a different threat model than "an adversarial agent could lie about what it touched".

**Hypothesis to test:** the higher-level abstraction (commit-time path declaration via enforced hook) is sound for the ring20 trust model, and the kernel-level mechanism's portability nightmare can be avoided entirely.

## 4. Re-scoped sub-arc proposal

Instead of building OS-level FS observation, file a sub-arc with three spikes:

### Spike T-2022a — git-hook path declaration

- Pre-commit hook on every AEF worktree posts `{agent_id, branch, paths_modified: [...], paths_added: [...], paths_deleted: [...]}` to a coordination topic.
- Hub maintains a sliding-window view: which agents have declared which paths in the last N minutes.
- Orchestrator queries this view before dispatching new work; on overlap, applies conservative policy (or, post-flip, evaluates against agreed merge rules).

**Cost:** ~80 LOC (hook + envelope + view-query verb). ≤1 session.

### Spike T-2022b — bypass detection

- Test: agent disables hook, commits anyway. Does the orchestrator notice?
- Mechanism: count commits per agent (via push-time observation or worktree state probe) and compare against count of declared path-sets. Divergence → alert.
- Sound for the "agent that forgets to announce" failure mode that §4 worried about.

**Cost:** ~50 LOC (probe verb + alert wiring). ≤1 session.

### Spike T-2022c — kernel-mechanism fallback

- ONLY if Spike T-2022a or T-2022b fail: revisit kernel-level mechanism for ring20-Linux-only subset of agents (drop macOS coverage as out-of-scope for now).
- Inotify (no caps needed) is the most viable fallback.

**Cost:** estimated 1-2 sessions, only conditional on prior spikes failing.

## 5. IW dispositions

- **IW-1 (mechanism — inotify/fanotify/ptrace/LD_PRELOAD/eBPF/FUSE):** RESOLVED as "none, at OS level". Re-scope to git-hook-enforced declaration. See artifact §2-§3. Confidence=4.
- **IW-2 (per-host viability):** RESOLVED — OS-level mechanisms fail on portability OR container caps. Git-hook approach is portable by construction (every host has git). Confidence=4.
- **IW-3 (blind spots):** For OS-level: catastrophic (every mechanism has a blind spot that breaks the §4 soundness). For git-hook: blind spot = unstaged scratch writes that never become commits — but those don't matter because they never persist to shared state. Confidence=3.
- **IW-4 (cost — per-syscall overhead, scaling):** OS-level: prohibitive for most mechanisms. Git-hook: one network post per commit, negligible. Confidence=4.
- **IW-5 (granularity — directory/file/byte-range, AEF needs):** File-level is sufficient. AEF needs to know "is anyone else touching path X" — directory and byte-range are overkill. Git already operates at file granularity. Confidence=4.

## 6. Recommendation

**DEFER as captured.** OS-level FS observation as a primitive is not viable in the ring20 deployment due to the empty mechanism-intersection (§2). However, the *concern* the primitive addresses (detect parallel writes to the same file for the conservative→optimistic flip) is real and worth resolving.

**Re-scope to a three-spike sub-arc (T-2022a/b/c).** Each spike is small and answers a separate question. If T-2022a + T-2022b succeed, the substrate gets the capability §6 #4 asks for at a different abstraction layer, and the conservative→optimistic flip becomes a tractable §4 conversation. If both fail, T-2022c falls back to kernel-mechanism on the Linux subset.

**Why not full NO-GO:** Conservative policy is correct today (§4) but expensive — agents serialize where they could parallelize. The ROI on cracking this is real, just not via the captured mechanism. NO-GO would close the question; DEFER + re-scoped spikes leaves it productively open.

**Why not GO with the re-scoped spikes immediately:** The Foundation primitives (T-2019..T-2021) need to land first so AEF can produce real workload evidence about how often conservative policy actually serializes work that could parallelize. Without that evidence, the spikes risk solving a problem that's smaller than expected.

## 7. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §4 capability/policy split | ✓ Either path (OS or git-hook) supplies capability; policy flip remains separate gated decision. |
| §4 "optimistic-on-honor-system is unsafe" | ✓ Re-scoped approach makes announcement structural-not-voluntary, addressing the honor-system critique. |
| §5 code plane / governance plane | ✓ Git-hook approach operates at the code-plane edge (commit boundary) where collisions matter most. |
| §9 SOFT dep | ✓ Co-discovered with AEF; this artifact's re-scoping is the kind of finding §9 SOFT-deps are designed for. |

## 8. Open follow-up tasks to file on DEFER + spike-arc start

- **Conditional, after Foundation primitives land + AEF produces serialization-cost evidence:**
  - T-2022a: git-hook path declaration spike (~80 LOC, ≤1 session)
  - T-2022b: bypass detection spike (~50 LOC, ≤1 session)
  - T-2022c: kernel-mechanism fallback (only if 2022a/b fail; 1-2 sessions, Linux-only subset)
- **Documentation:** add §3's "OS-level vs git-hook trade-off" reasoning to `docs/architecture/parallel-execution-substrate.md` as an addendum to §4.
