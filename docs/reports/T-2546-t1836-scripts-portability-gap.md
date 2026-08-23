# T-2546 — T-1836 MCP tools depend on repo scripts (Directive #4 portability gap)

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/active/T-2546-t-1836-mcp-tools-depend-on-repo-scripts-.md`.
> The exploration it describes was carried out earlier; this file relocates that
> trail into `docs/reports/` per C-001, it does not add findings.
>
> **Status at consolidation:** assumptions A-1..A-4 verified; all three IW
> questions **deferred**, awaiting a human product-scope call. The task remains
> `owner: human` and undecided. The open questions below are open — nothing here
> resolves them.

## The finding

TermLink's charter Directive #4 (Portability) forbids provider, language, and
environment lock-in. Verified in code, the T-1836 MCP tool family — roughly 18
tools, the listener-heartbeat / fleet-canary trio and siblings, added
"shell-out per PL-185" — resolves its scripts through `resolve_t1836_script`
(`crates/termlink-mcp/src/tools.rs:29098`), whose default directory is
**`/opt/termlink/scripts`**: this repository's dev-host checkout path. It then
runs them with `Command::new("bash")` (`tools.rs:29121`).

Two coupled facts follow:

1. `/opt/termlink/scripts` is a specific build-host absolute path baked into a
   distributed binary.
2. More fundamentally, **the scripts are not distributed with the installed
   binary at all.** A Homebrew or cargo consumer has no `scripts/` directory on
   disk, so *every one of these 18 MCP tools fails* until that consumer clones
   the repo and points `TERMLINK_SCRIPTS_DIR` at it.

The failure is **loud, not silent** — `resolve_t1836_script` returns a `json_err`
carrying a remediation hint. So this is a degraded-capability portability gap
rather than a crash, which is what keeps it an inception rather than a bug.

**Why it surfaced when it did:** the T-2468 purpose review was sweeping every
Constitutional Directive. Portability was the last un-run lens, and this is its
one genuine consumer-affecting finding.

## Assumptions — all verified

- **A-1** — `resolve_t1836_script` defaults to `/opt/termlink/scripts` and the
  tools shell out via `bash`. **VERIFIED** (`tools.rs:29098-29121`).
- **A-2** — the `scripts/` directory is not copied to any standard location by an
  install. **VERIFIED**: the release workflow copies only the binary
  (`.github/workflows/release.yml:53`) and the brew formula only `bin.install`s
  it (`homebrew/Formula/termlink.rb:39`). `scripts/` ships nowhere; a consumer
  install has zero scripts on disk.
- **A-3** — these ~18 tools were *intentionally* shell-outs to repo scripts
  (PL-185), i.e. designed as operator-host tooling run from a checkout, not as
  general-consumer MCP surface. **PLAUSIBLE** from the PL-185 note; the human owns
  whether that intent still holds.
- **A-4** — the core protocol/hub/session path has no comparable cross-OS
  portability defect. **VERIFIED** by the hunt: the runtime_dir resolution chain,
  the `/proc` soft-degrade, and the `setsid` fallback are all properly guarded.

A-4 is worth stating plainly because it bounds the finding: the portability gap is
confined to this one tool family, not the substrate.

## Open questions (all deferred to the human)

### IW-1 — Are the T-1836 canary/heartbeat MCP tools in scope for consumer (non-repo) installs at all, or are they operator-host-only by design?

*confidence 2 · deferred*

The PL-185 "shell-out to repo scripts" design implies operator-host intent, but
that is a product-scope call the human owns.

**This is the crux** — the answer decides whether any code change is warranted at
all. If operator-host-only, the remedy is documentation, not code. If
consumer-scoped, it opens IW-2.

### IW-2 — If consumer-scoped, which distribution strategy?

*confidence 1 · deferred*

Three candidates, with distinct cost and reversibility:

| Strategy | Shape | Cost |
|---|---|---|
| Embed in binary | `include_str!` → tempdir at call time | self-contained binary; moderate |
| Stage via installer | brew formula + release step copy to a resolved prefix | cross-repo change |
| Rewrite in-process | eliminate the bash shell-out entirely | large refactor; also retires the `bash` dependency |

This belongs to a GO build scope, not to this inception.

### IW-3 — Independent of scope, should the default resolution try a portable chain before the dev-host fallback?

*confidence 2 · deferred*

A small additive improvement: exe-relative → XDG data dir → `/opt/termlink/scripts`,
mirroring `runtime_dir()`'s existing chain, keeping both the env override and the
loud failure. It helps the "repo present but binary run from elsewhere" case even
if IW-1 answers operator-host-only. Low risk; a candidate for a bounded GO on its
own.
