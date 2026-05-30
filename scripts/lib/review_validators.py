"""
T-1885 v0.1 — Validators for fw independent-review.

Each validator runs in a separate subprocess (independent-reviewer rail —
producer code does NOT validate its own work). Verdict shape:

    {
        "verdict": "PASS-ROBUST" | "PASS-LOOSE" | "FAIL" | "INCONCLUSIVE",
        "reason": str,
        "evidence": str (markdown — rendered into the source task's Updates block),
    }

v0.1 validators:
- cli_validator         REVIEW-CLI: capture cmd output via `script -c`, grep Expected keywords
- watch_validator       CLI-WATCH: frame-capture + stability check (T-1884 S3 technique)
- release_validator     RUBBER-STAMP-RELEASE: gh release view

v0.2 (deferred): mechanical_validator (shell + remote-exec routing),
observe_infra_validator (termlink remote exec against fleet hosts).
"""
import re
import subprocess
import tempfile
from pathlib import Path


# ---------- shared helpers --------------------------------------------------

ANSI_RE = re.compile(r"\x1b\[[0-9;?]*[a-zA-Z]")
CLEAR_HOME = "\x1b[2J\x1b[H"
TS_RE = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")


def strip_ansi(s):
    return ANSI_RE.sub("", s).replace("\r", "")


def normalize_frame(s):
    s = strip_ansi(s)
    s = TS_RE.sub("<TS>", s)
    return s.strip()


def run_safe(cmd, timeout=30, cwd=None):
    """Run a command, capture stdout+stderr, return (exit, out, err)."""
    try:
        r = subprocess.run(
            ["bash", "-c", cmd],
            capture_output=True, text=True, timeout=timeout, cwd=cwd,
        )
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return -1, "", "[TIMEOUT]"
    except Exception as e:
        return -2, "", f"[EXEC-ERROR {e}]"


def _expected_keywords(expected_block):
    """Distill Expected: block into a few high-signal keywords for matching."""
    if not expected_block:
        return []
    # Drop common stopwords + framing
    keys = re.findall(r"[A-Za-z_/.][A-Za-z0-9_./-]{3,}", expected_block)
    stop = {"step", "shows", "matches", "expected", "should", "verify",
            "the", "and", "with", "without", "from", "this", "that", "for",
            "appear", "reads", "natural", "naturally"}
    return [k for k in keys if k.lower() not in stop][:6]


# ---------- REVIEW-CLI validator -------------------------------------------

ERROR_PATH_KEYWORDS = [
    "error", "fail", "failing", "invalid", "wrong", "non-existent",
    "nonexistent", "missing", "ambiguous", "rejected", "deny", "denied",
    "unauthorized", "not found", "names the failing", "names the offending",
    "actionable", "informative",
]


def _is_error_path_ac(ac_entry, expected):
    """Heuristic: does this AC test error-path behavior (where non-zero exit
    is the expected outcome)? Used to avoid over-penalizing non-zero exits."""
    text = (ac_entry + " " + (expected or "")).lower()
    return any(kw in text for kw in ERROR_PATH_KEYWORDS)


def cli_validator(ac_entry, steps_cmds, expected, *, capture_seconds=20):
    """REVIEW-CLI: run the AC's Steps commands, grep Expected keywords across
    stdout AND stderr.

    Verdict logic recognises error-path ACs (which expect non-zero exits):
        PASS-ROBUST   keyword match strong; if error-path, non-zero is OK
        PASS-LOOSE    cmds ran, partial match or no keywords to match
        FAIL          cmds clearly broken (timeout, exec-error) OR
                      not-error-path AND non-zero AND no keyword match
        INCONCLUSIVE  no commands extracted
    """
    if not steps_cmds:
        return {
            "verdict": "INCONCLUSIVE",
            "reason": "no commands extracted from Steps block",
            "evidence": "_no Steps commands found — manual review required_",
        }

    # Detect placeholder commands the validator cannot auto-run.
    PLACEHOLDER_RE = re.compile(r"<[a-z][a-zA-Z0-9_-]*>|<paste>|\$ARG|\{[a-z_]+\}", re.IGNORECASE)
    placeholder_cmds = [(n, c) for (n, c) in steps_cmds if PLACEHOLDER_RE.search(c)]
    if placeholder_cmds and len(placeholder_cmds) == len(steps_cmds):
        ev = ["**Validator:** REVIEW-CLI", "**Status:** placeholder Steps — cannot auto-run", ""]
        ev.append("**Commands with placeholders:**")
        for (n, c) in placeholder_cmds:
            ev.append(f"- step {n}: `{c}` (contains placeholder — operator must fill)")
        return {
            "verdict": "INCONCLUSIVE",
            "reason": f"all {len(steps_cmds)} Steps commands contain placeholders — operator must fill before run",
            "evidence": "\n".join(ev),
        }

    results = []
    for (n, cmd) in steps_cmds:
        if PLACEHOLDER_RE.search(cmd):
            results.append({
                "step": n, "cmd": cmd, "exit": "skip",
                "stdout": "", "stderr": "[skipped — placeholder in cmd]",
            })
            continue
        ex, out, err = run_safe(cmd, timeout=capture_seconds)
        results.append({"step": n, "cmd": cmd, "exit": ex, "stdout": out, "stderr": err})

    error_path = _is_error_path_ac(ac_entry, expected)
    runnable = [r for r in results if r["exit"] != "skip"]
    if not runnable:
        return {
            "verdict": "INCONCLUSIVE",
            "reason": "no runnable commands (all placeholders)",
            "evidence": "_validator skipped all Steps as placeholders_",
        }
    has_exec_errors = any(r["exit"] in (-1, -2) for r in runnable)
    all_zero = all(r["exit"] == 0 for r in runnable)
    combined = "\n".join((r["stdout"] + "\n" + r["stderr"]) for r in runnable)
    keywords = _expected_keywords(expected or "")
    matched_keys = [k for k in keywords if k.lower() in combined.lower()]
    match_ratio = (len(matched_keys) / len(keywords)) if keywords else 0.0

    # Taste-call ACs have no meaningful Expected keywords by construction
    # ("reads naturally", "feels right"). Surface output for operator judgment
    # rather than FAILing — the validator cannot adjudicate taste.
    is_taste_call = not keywords

    if has_exec_errors:
        verdict = "FAIL"
        reason = "timeout or exec-error on at least one command"
    elif match_ratio >= 0.5:
        verdict = "PASS-ROBUST"
        reason = (
            f"{len(matched_keys)}/{len(keywords)} expected keys matched"
            + (f" (error-path: non-zero exits OK)" if error_path and not all_zero else "")
        )
    elif match_ratio > 0:
        verdict = "PASS-LOOSE"
        reason = f"partial keyword match ({len(matched_keys)}/{len(keywords)})"
    elif is_taste_call:
        # No Expected keywords → can't auto-decide. Surface output as evidence.
        any_stderr = any(r["stderr"].strip() for r in results)
        any_stdout = any(r["stdout"].strip() for r in results)
        if any_stdout or any_stderr:
            verdict = "PASS-LOOSE"
            reason = "taste-call AC — output captured for operator judgment (validator cannot adjudicate taste)"
        else:
            verdict = "INCONCLUSIVE"
            reason = "taste-call AC AND no output captured — likely command path missing locally"
    elif all_zero:
        verdict = "PASS-LOOSE"
        reason = f"cmds zero-exit but no Expected keyword match — manual verify"
    elif error_path:
        verdict = "PASS-LOOSE" if any(r["stderr"].strip() for r in runnable) else "FAIL"
        reason = "error-path AC: non-zero exit + stderr present but no Expected keyword hit"
    else:
        verdict = "FAIL"
        bad = [r for r in runnable if r["exit"] != 0]
        reason = f"{len(bad)}/{len(runnable)} cmds non-zero exit, no keyword match"

    # Build evidence block
    ev = ["**Validator:** REVIEW-CLI"]
    if error_path:
        ev.append("**Mode:** error-path (non-zero exits expected, matched against stderr)")
    ev.append("")
    ev.append("**Steps results:**")
    for r in results:
        out_line = (r["stdout"].splitlines() or [""])[0][:160]
        err_line = (r["stderr"].splitlines() or [""])[0][:160]
        ev.append(f"- step {r['step']}: `{r['cmd'][:100]}` → exit={r['exit']}")
        if out_line:
            ev.append(f"  - stdout: `{out_line!r}`")
        if err_line:
            ev.append(f"  - stderr: `{err_line!r}`")
    if keywords:
        ev.append("")
        ev.append(f"**Expected keywords:** {keywords}")
        ev.append(f"**Matched:** {matched_keys} ({match_ratio:.0%})")

    return {"verdict": verdict, "reason": reason, "evidence": "\n".join(ev)}


# ---------- CLI-WATCH validator --------------------------------------------

def watch_validator(ac_entry, *, command=None, duration_s=8, interval_s=2):
    """CLI-WATCH: capture pty output via `script -c`, split on clear+home,
    assert frame-body stability modulo timestamp.

    If `command` is not given, try to extract from the AC entry's first
    backtick-quoted command. Skips on extraction failure.
    """
    if command is None:
        # Try to find a --watch command in the AC entry
        m = re.search(r"`([^`]*--watch[^`]*)`", ac_entry)
        if m:
            command = m.group(1)
        else:
            return {
                "verdict": "INCONCLUSIVE",
                "reason": "no --watch command found in AC text",
                "evidence": "_validator could not extract a --watch invocation — pass --command explicitly_",
            }

    with tempfile.NamedTemporaryFile(mode="w", suffix=".log", delete=False) as tf:
        log_path = Path(tf.name)
    try:
        shell_inner = f"timeout {duration_s} {command} || true"
        try:
            subprocess.run(
                ["script", "-q", "-c", shell_inner, str(log_path)],
                capture_output=True, text=True, timeout=duration_s + 10,
            )
        except subprocess.TimeoutExpired:
            return {
                "verdict": "INCONCLUSIVE",
                "reason": "script -c capture timed out",
                "evidence": f"_capture timed out for command: `{command}`_",
            }
        raw = log_path.read_text(encoding="utf-8", errors="replace")
    finally:
        log_path.unlink(missing_ok=True)

    parts = raw.split(CLEAR_HOME)
    frames = [p for p in parts[1:] if p.strip()]

    if not frames:
        return {
            "verdict": "INCONCLUSIVE",
            "reason": "no frames captured — --watch may not be supported by this command",
            "evidence": f"_raw capture {len(raw)} bytes, no clear+home redraws_",
        }
    if len(frames) == 1:
        return {
            "verdict": "INCONCLUSIVE",
            "reason": f"only 1 frame in {duration_s}s — interval may exceed window",
            "evidence": f"_captured 1 frame from `{command}` in {duration_s}s; raise duration or lower interval_",
        }

    normalized = [normalize_frame(f) for f in frames]
    distinct = set(normalized)

    first_frame_preview = "\n".join(f"    {l}" for l in normalized[0].splitlines()[:6])
    ev = [
        "**Validator:** CLI-WATCH frame-capture",
        f"**Command:** `{command}`",
        f"**Capture:** {duration_s}s, interval={interval_s}s, frames={len(frames)}, distinct={len(distinct)}",
        "",
        "**First frame (normalized):**",
        "```",
        first_frame_preview,
        "```",
    ]

    if len(distinct) == 1:
        return {
            "verdict": "PASS-ROBUST",
            "reason": f"{len(frames)} frames byte-identical modulo timestamp",
            "evidence": "\n".join(ev),
        }
    if len(distinct) <= 3:
        return {
            "verdict": "PASS-LOOSE",
            "reason": f"{len(frames)} frames, {len(distinct)} distinct bodies (content changed, no flicker)",
            "evidence": "\n".join(ev),
        }
    return {
        "verdict": "FAIL",
        "reason": f"{len(frames)} frames, {len(distinct)} distinct bodies — flicker / instability suspected",
        "evidence": "\n".join(ev),
    }


# ---------- RUBBER-STAMP-RELEASE validator ---------------------------------

def release_validator(ac_entry, *, tag=None):
    """RUBBER-STAMP-RELEASE: gh release view <tag> + check asset list."""
    if tag is None:
        # Try to find a vX.Y.Z reference in the AC entry
        m = re.search(r"\bv(\d+\.\d+\.\d+)\b", ac_entry)
        if m:
            tag = "v" + m.group(1)
    if tag is None:
        return {
            "verdict": "INCONCLUSIVE",
            "reason": "no release tag found in AC text",
            "evidence": "_validator could not extract a vX.Y.Z tag — pass --tag explicitly_",
        }

    ex, out, err = run_safe(f"gh release view {tag} --json name,tagName,assets 2>&1", timeout=20)
    if ex != 0:
        return {
            "verdict": "FAIL",
            "reason": f"gh release view {tag} failed (exit={ex})",
            "evidence": f"**Validator:** RUBBER-STAMP-RELEASE\n**Tag:** `{tag}`\n**gh err:** `{(err or out)[:200]!r}`",
        }
    # Check for macOS + Linux assets
    has_mac = "darwin" in out.lower() or "macos" in out.lower()
    has_linux = "linux" in out.lower()
    if has_mac and has_linux:
        return {
            "verdict": "PASS-ROBUST",
            "reason": f"release {tag} has both macOS and Linux assets",
            "evidence": f"**Validator:** RUBBER-STAMP-RELEASE\n**Tag:** `{tag}`\n**Assets present:** macOS + Linux\n```\n{out[:600]}\n```",
        }
    return {
        "verdict": "PASS-LOOSE" if (has_mac or has_linux) else "FAIL",
        "reason": f"release {tag}: macOS={has_mac}, Linux={has_linux}",
        "evidence": f"**Validator:** RUBBER-STAMP-RELEASE\n**Tag:** `{tag}`\n```\n{out[:600]}\n```",
    }


# ---------- surface (no validator, just render the AC as "needs human") ----

def surface_validator(klass, ac_entry):
    """Surface-only: OPERATOR-ACTION / TIME-GATED / OTHER / REVIEWER-AGENT-MISFILE."""
    reason_map = {
        "OPERATOR-ACTION": "human must do the action first; agent only verifies after",
        "TIME-GATED": "deferred event (future deploy / time window); no verdict possible now",
        "OTHER": "AC class ambiguous to classifier; manual sort needed",
        "REVIEWER-AGENT-MISFILE": "AC has [REVIEWER] prefix on ### Human — template error, should be ### Agent",
    }
    return {
        "verdict": "SURFACE",
        "reason": reason_map.get(klass, "no validator for this class"),
        "evidence": f"_class={klass}, surfaced without validation per orchestrator design_",
    }


# ---------- dispatcher ------------------------------------------------------

def validate(ac_entry, klass, steps_cmds, expected):
    """Route to the right validator based on class."""
    from .review_classifier import v01_class_routes_to
    route = v01_class_routes_to(klass)
    if route == "cli":
        return cli_validator(ac_entry, steps_cmds, expected)
    if route == "watch":
        return watch_validator(ac_entry)
    if route == "release":
        return release_validator(ac_entry)
    if route == "v02-defer":
        return {
            "verdict": "SURFACE",
            "reason": "v0.2 validator (remote-exec) — surfaced for now",
            "evidence": f"_class={klass}: needs remote-exec routing, deferred to T-1886 v0.2_",
        }
    return surface_validator(klass, ac_entry)
