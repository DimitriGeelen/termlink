#!/usr/bin/env python3
"""Render docs/reports/arc011-ph1-acceptance.md from one B14 run (arc-011 PH-1).

Inputs come from scripts/arc011-ph1-demo.sh: the acceptance test's evidence JSON,
the `strace -f` log of the whole run (the X4 path audit), and the floor-probe
results. Exit 0 when X1, X3 and X4 all pass, 1 otherwise, 2 on unusable input.
"""
import argparse
import json
import re
import sys
from collections import Counter, defaultdict

STAGES = ["SENT", "RECEIVED", "STORED", "HANDED_OVER", "REPLIED"]
FLOOR = ["SI-3", "SI-4", "SI-6", "SI-7", "SI-8", "SI-14", "SI-15", "SI-16", "SI-17", "SI-18",
         "SI-22", "SI-23", "SI-25", "SI-26", "SI-30", "FX-2", "FX-7"]
FLOOR_TEXT = {
    "SI-3": "every message signed by the sender's own key, verified at hub and receiver",
    "SI-4": "same (sender, conversation, id) with another digest is a conflict",
    "SI-6": "stage records signed by the recorder, forward-only",
    "SI-7": "peer content only inside a receiver-built frame with a random boundary",
    "SI-8": "no message grants, approves or raises trust",
    "SI-14": "owner-only send/status sockets, admin separate",
    "SI-15": "STORED only after a durable, checksummed write; estate guard",
    "SI-16": "one sidecar per store, one consumer per inbox",
    "SI-17": "HANDED_OVER only on the nonce in a harness-written record",
    "SI-18": "priority, urgency and trust decided at the receiver; allow-list; downgrade recorded",
    "SI-22": "per-conversation hash chain, breaks named",
    "SI-23": "operator switch for mid-turn delivery, audited, read every call",
    "SI-25": "no secret in any log, status, frame, record or hub post",
    "SI-26": "a message for an exact instance reaches only that instance",
    "SI-30": "one atomic claim per attempt; redeliver only on proof of absence",
    "FX-2": "declared hub id verified; wrong hub refused",
    "FX-7": "nothing says delivered without HANDED_OVER",
}
# Explanations for attempts seen on this host; matched by substring, shown beside the line.
KNOWN_ATTEMPTS = [
    ("/home/claude/.claude/remote/", "Claude Code looks for a cloud-session credential at a fixed path at start; it does not exist here"),
    ("openat /var/lib/termlink, by /opt/termlink/target/", "the test kit's own before/after snapshot of production paths; the unprivileged account may not read it"),
]
TWO_PATH = {"rename", "renameat", "renameat2", "link", "linkat", "symlink", "symlinkat"}
QUOTED = re.compile(r'"((?:[^"\\]|\\.)*)"')
LINE = re.compile(r"^(\d+)\s+(.*)$")


def utc(ms):
    import datetime
    return datetime.datetime.fromtimestamp(ms / 1000, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z"


def parse_strace(path):
    """Return (syscall, [paths], ok, who, err) for each path syscall and connect.

    `who` is the program the process was running at the time (from execve, inherited
    across clone/fork); `err` the errno name of a failed call."""
    pending = {}
    exe = {}
    out = []
    with open(path, errors="replace") as f:
        for raw in f:
            m = LINE.match(raw.rstrip("\n"))
            if not m:
                continue
            pid, rest = m.groups()
            if rest.startswith("<... "):
                name = rest[5:].split(" ", 1)[0]
                start = pending.pop((pid, name), None)
                if start is None:
                    continue
                rest = start + rest.split("resumed>", 1)[-1]
            elif rest.endswith("<unfinished ...>"):
                name = rest.split("(", 1)[0]
                pending[(pid, name)] = rest[: -len("<unfinished ...>")]
                continue
            if rest.startswith(("+++", "---")):
                continue
            name = rest.split("(", 1)[0]
            ret = rest.rsplit(" = ", 1)[-1] if " = " in rest else "?"
            ok = bool(re.match(r"^\d", ret))
            err = ret.split()[1] if ret.startswith("-1 ") and len(ret.split()) > 1 else ""
            who = exe.get(pid, "?")
            if name in ("clone", "clone3", "fork", "vfork"):
                child = re.match(r"^(\d+)", ret)
                # A vfork-style child may exec before its parent's clone returns:
                # never overwrite what the child already ran.
                if child:
                    exe.setdefault(child.group(1), who)
                continue
            if name in ("execve", "execveat"):
                q = QUOTED.findall(rest)
                if ok and q:
                    exe[pid] = q[0]
                out.append((name, q[:1], ok, who, err))
                continue
            if name == "connect":
                if "sun_path=" in rest:
                    p = re.search(r'sun_path=(?:@)?"((?:[^"\\]|\\.)*)"', rest)
                    tgt = ("unix", p.group(1) if p else "?")
                else:
                    port = re.search(r"sin6?_port=htons\((\d+)\)", rest)
                    addr = re.search(r'inet_(?:addr|pton)\((?:AF_INET6, )?"([^"]+)"', rest)
                    if not port:
                        continue
                    tgt = ("inet", f"{addr.group(1) if addr else '?'}:{port.group(1)}")
                # EINPROGRESS is a non-blocking connect under way, not a refusal.
                out.append(("connect", [tgt], ok or err == "EINPROGRESS", who, err))
                continue
            q = QUOTED.findall(rest)
            if not q:
                continue
            paths = q[:2] if name in TWO_PATH else q[:1]
            out.append((name, paths, ok, who, err))
    return out


def classify(p, estate, user, home):
    if p.startswith(estate + "/") or p == estate:
        return "estate"
    prod = (
        p == "/root" or p.startswith("/root/")
        or p.startswith("/var/lib/termlink")
        or re.match(r"^/tmp/termlink-\d+(/|$)", p)
        or p.startswith("/run/user/0")
        or (p.startswith("/opt/termlink") and not p.startswith("/opt/termlink/target/"))
        or (p.startswith("/home/") and not (p == home or p.startswith(home + "/")))
        or p.startswith("/etc/termlink")
    )
    if prod:
        return "PRODUCTION"
    if p == home or p.startswith(home + "/"):
        return f"agent home ({user})"
    if p.startswith("/opt/termlink/target/"):
        return "build output (binaries under test)"
    if not p.startswith("/"):
        return "relative (cwd: the run dir or an estate project)"
    if p.startswith("/tmp/"):
        return "/tmp outside the estate"
    if p == "/" or p.startswith("/System/"):  # the root dir itself; node's macOS cert probe
        return "system"
    for sysd in ("/usr/", "/lib", "/bin", "/sbin", "/etc/", "/proc/", "/sys/", "/dev/", "/run/", "/var/", "/snap/"):
        if p.startswith(sysd) or p == sysd.rstrip("/"):
            return "system"
    return "other"


def main():
    ap = argparse.ArgumentParser()
    for a in ("evidence", "strace", "probes", "acceptance-log", "acceptance-rc", "probes-run", "agent-user",
              "agent-home", "repo", "started", "head", "out", "evidence-out"):
        ap.add_argument("--" + a, required=True)
    a = ap.parse_args()

    try:
        ev = json.load(open(a.evidence))
    except (OSError, ValueError) as e:
        ev = {"steps": [], "load_error": str(e)}
    estate = ev.get("estate_root", "/nonexistent-estate")
    acc_rc = int(a.acceptance_rc)
    steps = {s["bv"]: s for s in ev.get("steps", [])}

    # ---- X1: BV-1..BV-7 --------------------------------------------------------------
    bv_ok = {f"BV-{i}": steps.get(f"BV-{i}", {}).get("ok") is True for i in range(1, 8)}
    x1 = all(bv_ok.values()) and acc_rc == 0
    # OQ-2 ruling E (T-3418): a third agent outside B's roster knocks, is held, the
    # operator admits it through the admin socket, and B gets it at its next tool call.
    oq2_ok = steps.get("OQ-2", {}).get("ok") is True

    # ---- X4: the strace audit ---------------------------------------------------------
    try:
        events = parse_strace(a.strace)
    except OSError as e:
        print(f"arc011-acceptance-report: cannot read the strace log: {e}", file=sys.stderr)
        return 2
    by_class = defaultdict(set)
    prod_ok, prod_failed = set(), set()
    calls = Counter()
    unix_targets, inet_targets = defaultdict(set), Counter()
    for name, paths, ok, who, err in events:
        calls[name] += 1
        if name == "connect":
            kind, tgt = paths[0]
            if kind == "inet":
                inet_targets[tgt] += 1
                if tgt.endswith(":9100"):
                    (prod_ok if ok else prod_failed).add(f"connect inet {tgt} (production hub port), by {who}{', ' + err if err else ''}")
                continue
            c = classify(tgt, estate, a.agent_user, a.agent_home)
            unix_targets[c].add(tgt)
            if c == "PRODUCTION":
                (prod_ok if ok else prod_failed).add(f"connect unix {tgt}, by {who}{', ' + err if err else ''}")
            continue
        for p in paths:
            c = classify(p, estate, a.agent_user, a.agent_home)
            by_class[c].add(p)
            if c == "PRODUCTION":
                (prod_ok if ok else prod_failed).add(f"{name} {p}, by {who}{', ' + err if err else ''}")
    x4 = not prod_ok and bool(events) and estate != "/nonexistent-estate"

    # ---- X3: floor probes --------------------------------------------------------------
    probes = []
    for line in open(a.probes):
        parts = line.rstrip("\n").split("\t")
        if len(parts) == 5:
            probes.append(dict(zip(("item", "target", "name", "result", "secs"), parts)))
    per_item = {i: [p for p in probes if p["item"] == i] for i in FLOOR}
    item_ok = {i: bool(ps) and all(p["result"] == "pass" for p in ps) for i, ps in per_item.items()}
    x3 = a.probes_run == "1" and all(item_ok.values())

    scan = ev.get("secret_scan_hits")
    survivors = ev.get("surviving_processes")
    live_ok = scan == [] and survivors == []
    verdict = x1 and oq2_ok and x3 and x4 and live_ok

    # ---- render --------------------------------------------------------------------------
    L = []
    w = L.append
    w("# arc-011 PH-1 acceptance: real agents (B14, with the OQ-2 ruling E from B15)")
    w("")
    w("Task T-3417 (B14). Exits X1, X3 and X4 of `docs/design/interactive-agent-communication-03-floor-and-phasing.md`")
    w("section 5.9; architecture note `docs/design/arc-011-ph1-architecture.md` table 13 row B14. Generated by")
    w("`scripts/arc011-ph1-demo.sh` (run as root; the run itself is unprivileged). Do not edit by hand; re-run the script.")
    w("")
    w(f"**Verdict: {'PASS' if verdict else 'FAIL'}.** Run started {a.started} on commit {a.head}.")
    w("")
    w("| Exit | What | Result |")
    w("|---|---|---|")
    w(f"| X1 | BV-1..BV-7 from a clean state; the record of m1 shows the five rows with times | {'pass' if x1 else 'FAIL'} |")
    w(f"| OQ-2 E | a stranger is held at the receiver, the operator decides, the sender always hears back (ruling 2026-10-12) | {'pass' if oq2_ok else 'FAIL'} |")
    w(f"| X3 | the 17 floor items of PH-1, each by its tests | {'pass' if x3 else ('not run' if a.probes_run != '1' else 'FAIL')} |")
    w(f"| X4 | no production credential, inbox, hub or pin touched (strace of the whole run) | {'pass' if x4 else 'FAIL'} |")
    w(f"| SI-25 live | the secret canary in nothing the estate or the agents wrote | {'pass' if scan == [] else f'FAIL {scan}'} |")
    w(f"| teardown | no estate process survived | {'pass' if survivors == [] else f'FAIL {survivors}'} |")
    w("| X6 | the operator has seen the demonstration run | the operator's; record it in T-3352 |")
    w("")
    w("## The run")
    w("")
    w(f"1. Account: `{a.agent_user}` (uid {ev.get('uid')}), HOME `{ev.get('home')}`. The test refuses to run as root (E-7).")
    w(f"2. Agents: `{ev.get('claude')}`, {ev.get('claude_version')}, model `{ev.get('model')}`, `claude -p` with")
    w("   `--setting-sources \"\"` (no user or project settings), the PostToolUse, SessionStart and Stop hooks given by")
    w("   `--settings`, and `--allowedTools` naming only the one command each step needs. Each agent's PATH carries")
    w("   only `termlink-sidecar` from the build.")
    w(f"3. Estate: `{estate}` (deleted at the end): its own hub (id `{ev.get('hub_id')}`), keys, runtime dir and")
    w("   stores; the two sidecars run with HOME inside it. Claude Code writes the agents' transcripts under the")
    w("   account's own `~/.claude/projects/`; the sidecars are told that directory (`claude_projects`), and the two")
    w("   project directories the run created there are deleted with the estate.")
    if ev.get("started_ms") and ev.get("finished_ms"):
        w(f"4. Duration: {(ev['finished_ms'] - ev['started_ms']) / 1000:.0f} s.")
    w("")
    w("| Step | Result | Evidence |")
    w("|---|---|---|")

    def d(bv):
        return steps.get(bv, {}).get("detail", {})

    b1 = d("BV-1")
    w(f"| BV-1 sidecars up, status answers | {'pass' if bv_ok['BV-1'] else 'FAIL'} | both name hub `{(b1.get('agent-a') or {}).get('hub_id')}` and {len((b1.get('agent-a') or {}).get('missing_strength') or [])} missing strength flags; delivery `{(b1.get('agent-a') or {}).get('delivery')}` |")
    b2 = d("BV-2")
    w(f"| BV-2 A hands m1 to its sidecar | {'pass' if bv_ok['BV-2'] else 'FAIL'} | A ran `{(b2.get('bash_commands') or ['?'])[0]}`; msg id `{b2.get('msg_id')}`; A said `{b2.get('claude_stdout')}` ({b2.get('claude_secs')} s) |")
    b3 = d("BV-3")
    w(f"| BV-3 signed post on the hub | {'pass' if bv_ok['BV-3'] else 'FAIL'} | offset {b3.get('hub_offset')} of `{b3.get('hub_topic')}`; the same post altered after signing: `{b3.get('altered_copy_refused_with')}` |")
    b4 = d("BV-4")
    w(f"| BV-4 B's sidecar receives and stores | {'pass' if bv_ok['BV-4'] else 'FAIL'} | A shows RECEIVED and STORED with B's times, label `{b4.get('label_a')}`; body found in `{', '.join(b4.get('body_in_b_store') or []) or 'nothing'}` |")
    b5 = d("BV-5")
    w(f"| BV-5 hand-over at B's next tool call | {'pass' if bv_ok['BV-5'] else 'FAIL'} | the frame is in a `hook_additional_context` record of B's own transcript; HANDED_OVER at B: {b5.get('handed_over_at_b')}, seen by A: {b5.get('seen_by_a')}; B said `{b5.get('claude_stdout')}` |")
    b6 = d("BV-6")
    w(f"| BV-6 B replies; A gets the reply | {'pass' if bv_ok['BV-6'] else 'FAIL'} | B ran `{(b6.get('bash_commands_b') or ['?', '?'])[-1]}`; REPLIED on m1: {b6.get('replied_on_m1')}; reply `{b6.get('reply_msg_id')}` handed to A at its next tool call |")
    w(f"| BV-7 A prints its record | {'pass' if bv_ok['BV-7'] else 'FAIL'} | five rows with times in order (below) |")
    w("")
    w("### BV-7: A's record for m1")
    w("")
    rec = d("BV-7").get("record") or []
    w("| Stage | Time (UTC) | Recorded by |")
    w("|---|---|---|")
    for r in rec:
        t = utc(r["at_ms"]) if isinstance(r.get("at_ms"), int) else "(absent)"
        w(f"| {r.get('stage')} | {t} | {r.get('by', '')} |")
    chain = d("BV-7").get("chain") or {}
    w("")
    w(f"Chain of conversation c1 at A: intact {chain.get('intact')}, {chain.get('len')} lines, head `{chain.get('head')}`.")
    w("")
    w("What agent A printed when it ran `termlink-sidecar status --msg-id <m1>` in its resumed session:")
    w("")
    w("```")
    w((b6.get("claude_a2_stdout") or "(nothing)").replace("```", "'''"))
    w("```")
    w("")
    w("### BV-5: the frame as it sits in B's transcript")
    w("")
    w("```")
    w((b5.get("frame") or "(no frame)").replace("```", "'''"))
    w("```")
    w("")
    w("B's transcript, record by record: " + ", ".join(f"`{k}`" for k in (b5.get("transcript_records") or [])) + ".")
    w("")
    w("A's reply frame (BV-6), as it sits in A's transcript:")
    w("")
    w("```")
    w((b6.get("reply_frame_in_a") or "(no frame)").replace("```", "'''"))
    w("```")
    w("")
    q = d("OQ-2")
    w("### OQ-2 ruling E, live: a third agent knocks")
    w("")
    w("A third real Claude session, agent-c, whose key is not on agent-b's roster, sends agent-b a question with a fake")
    w("\"operator pass\" in the text. The operator's ruling (T-3352, 2026-10-12): the receiving side holds the message and asks")
    w("the operator itself; only the operator admits, through the admin socket, never through a message; a pass inside a")
    w("message counts for nothing; the sender always hears back.")
    w("")
    w("| Step | Result | Evidence |")
    w("|---|---|---|")
    w(f"| agent-c sends through its Bash tool | {'pass' if q.get('msg_id') else 'FAIL'} | msg id `{q.get('msg_id')}`, key `{q.get('stranger_key')}` |")
    w(f"| held at agent-b, not refused | {'pass' if q.get('held_at_b') else 'FAIL'} | agent-b's status labels it HELD FOR OPERATOR |")
    w(f"| agent-b's side asks the operator itself | {'pass' if q.get('operator_asked') else 'FAIL'} | needs-attention entry, quoted below |")
    w(f"| the sender hears the knock | {'pass' if q.get('sender_heard_held') else 'FAIL'} | agent-c's status labels it HELD BY RECIPIENT (awaiting their operator), not UNKNOWN |")
    w(f"| a real agent-b turn while held gets nothing | {'pass' if q.get('not_delivered_while_held') else 'FAIL'} | agent-b ran `cat TASK.md`; no frame for the message in its transcript |")
    w(f"| the operator allows, on the admin socket (the test stands in for the operator) | {'pass' if q.get('allow_rc') == 0 else 'FAIL'} | `termlink-sidecar admin allow --key {q.get('stranger_key')} --why ...` answered `{q.get('allow_answer')}` |")
    w(f"| the sender hears it was admitted | {'pass' if q.get('sender_heard_admitted') else 'FAIL'} | agent-c's label becomes WAITING FOR RECIPIENT |")
    w(f"| agent-b gets it at its next tool call and answers | {'pass' if q.get('handed_over') and q.get('replied') else 'FAIL'} | HANDED_OVER {q.get('handed_over')}, REPLIED {q.get('replied')}; agent-b ran `{(q.get('bash_commands_b') or ['?'])[-1]}` |")
    w("")
    w("agent-c's record of its message, in order: " + ", ".join(f"`{r}`" for r in (q.get("sender_rows") or [])) + ".")
    w("")
    w("The question agent-b's sidecar raised to the operator:")
    w("")
    w("```")
    for a_line in q.get("operator_asked") or []:
        try:
            w(json.loads(a_line).get("detail", a_line))
        except ValueError:
            w(a_line)
    w("```")
    w("")
    w("The decision in agent-b's `_admin` log:")
    w("")
    w("```")
    for r in q.get("admin_log") or []:
        w(f"{r.get('stage')}  {utc(r['ts_ms']) if isinstance(r.get('ts_ms'), int) else ''}  by {r.get('sender')}  {r.get('reason')}")
    w("```")
    w("")
    w("The frame agent-b received after the operator allowed it (note `trust=admitted-by-operator`; the pass in the text")
    w("stayed text):")
    w("")
    w("```")
    w((q.get("frame") or "(no frame)").replace("```", "\'\'\'"))
    w("```")
    w("")
    he = (b5.get("hook_errors_b") or "") + (ev.get("hook_errors_a") or "")
    w(f"Hook errors logged: {'none' if not he.strip() else he.strip()}.")
    w("")

    w("## X4: what the run touched outside the estate")
    w("")
    w("`strace -f` traced every process of the run: the test, the hub, the three sidecars, all Claude Code processes,")
    w("their tools, the hooks and the detached finalizers. Syscalls traced: open/openat/openat2/creat, execve,")
    w("rename, link, symlink, unlink, mkdir, truncate, chmod, chown and connect. A path is **production** when it is")
    w("root's HOME (`/root`, which holds `~/.termlink`, the pins and the operator's Claude login), `/var/lib/termlink`")
    w("(the production hub), `/tmp/termlink-<uid>`, `/run/user/0`, the repository outside `target/` (its `.context`,")
    w("`.tasks`, inbox state), `/etc/termlink`, another account's HOME, or a connection to port 9100 (the production hub).")
    w("")
    w(f"Events parsed: {sum(calls.values())} ({', '.join(f'{k} {v}' for k, v in calls.most_common())}).")
    w("")
    w(f"**Production paths opened or changed: {len(prod_ok)}.** Production paths attempted and refused by the kernel or absent: {len(prod_failed)}.")
    w("")
    for p in sorted(prod_ok):
        w(f"- OPENED: `{p}`")
    for p in sorted(prod_failed)[:50]:
        note = next((n for k, n in KNOWN_ATTEMPTS if k in p), "")
        w(f"- attempted, failed: `{p}`" + (f" ({note})" if note else ""))
    if prod_ok or prod_failed:
        w("")
    w("| Class | Distinct paths |")
    w("|---|---|")
    for c in sorted(by_class, key=lambda c: (c != "estate", c)):
        w(f"| {c} | {len(by_class[c])} |")
    w("")
    for c in sorted(by_class):
        if c in ("estate", "system", "PRODUCTION"):
            continue
        items = sorted(by_class[c])
        w(f"<details><summary>{c}: {len(items)} paths</summary>")
        w("")
        for p in items[:400]:
            w(f"- `{p}`")
        if len(items) > 400:
            w(f"- ... {len(items) - 400} more")
        w("")
        w("</details>")
        w("")
    w("Socket connections: " + "; ".join(
        [f"unix {c}: {len(v)}" for c, v in sorted(unix_targets.items())]
        + [f"inet {t} x{n}" for t, n in sorted(inet_targets.items())]) + ".")
    w("")
    w("The estate guard part of X4 (a sidecar started against a non-throwaway path refuses; a record failing its")
    w("checksum is never reported STORED) is probed under SI-15 below.")
    w("")

    w("## X3: the 17 floor items")
    w("")
    if a.probes_run != "1":
        w("Not run (`--no-probes`).")
    else:
        w("Each item's tests ran one at a time as root (`fc_2` mounts a tmpfs; `api_real_other_uid_cannot_reach_the_socket`")
        w("switches uid). The negative control of each fail-closed row is kill-checked separately")
        w("(`docs/reports/arc011-ph1-fc-matrix.md`, 18 of 18 killed).")
        w("")
        w("| # | Item | What it holds | Tests | Result |")
        w("|---|---|---|---|---|")
        for n, i in enumerate(FLOOR, 1):
            tests = "<br>".join(f"`{p['target']}::{p['name']}` {p['result']} ({p['secs']} s)" for p in per_item[i]) or "none"
            w(f"| {n} | {i} | {FLOOR_TEXT[i]} | {tests} | {'pass' if item_ok[i] else 'FAIL'} |")
        w("")
        w(f"{sum(item_ok.values())} of {len(FLOOR)} items pass. SI-18 includes the allow-list (architecture note 9.4).")
    w("")
    w("## Limits of this run, stated plainly")
    w("")
    w("1. Both agents run as the one account `testdev`, so the sidecars' owner check separates them from root and")
    w("   other accounts but not from each other (the status call says so: \"no per-agent accounts\"). Track A.")
    w("2. The sidecars are not supervised; a crashed sidecar is restarted by nothing in PH-1.")
    w("3. `claude -p` runs one turn per call; the BV shows delivery at the next tool call (OP-21 B), not a typed doorbell")
    w("   into an idle prompt (PH-2).")
    w("4. X6 (the operator has watched it run) is the operator's, recorded in T-3352; the move to PH-2 is the operator's decision.")
    w("")
    w("## Reproduce")
    w("")
    w("```")
    w("sudo bash scripts/arc011-ph1-demo.sh            # this report")
    w("sudo bash scripts/arc011-ph1-demo.sh --keep     # also keep the strace log and probe logs")
    w("```")
    w("")
    with open(a.out, "w") as f:
        f.write("\n".join(L))

    ev_out = dict(ev)
    ev_out["x4"] = {"production_opened": sorted(prod_ok), "production_attempted": sorted(prod_failed),
                    "classes": {c: len(v) for c, v in by_class.items()}, "events": dict(calls)}
    ev_out["probes"] = probes
    ev_out["verdict"] = {"x1": x1, "x3": x3, "x4": x4, "live_scan_and_teardown": live_ok, "pass": verdict}
    with open(a.evidence_out, "w") as f:
        json.dump(ev_out, f, indent=2)
        f.write("\n")
    print(f"arc011-acceptance-report: X1 {x1} X3 {x3} X4 {x4} live {live_ok} -> {'PASS' if verdict else 'FAIL'}")
    return 0 if verdict else 1


if __name__ == "__main__":
    sys.exit(main())
