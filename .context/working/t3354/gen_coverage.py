#!/usr/bin/env python3
"""T-3354 working tool: build the coverage tables of 05a from the spec headers of 05.

Reads docs/design/interactive-agent-communication-05-evidence.md, collects for every spec E-n the ids
it links (header "Links:" for section 6-9 specs, the "Links" column for the section 10 table, and the
AFC table of section 13), and prints Markdown tables: R-1..R-82, SI-1..SI-31, FX-1..FX-8, the
"impossible" requirements, AFC-n, the ARC4 15 measured components. Exit 0 if every item has a spec or a
declared gap, 1 if any item is uncovered and undeclared, 2 on a parse problem.
"""
import re
import sys

DOC = "docs/design/interactive-agent-communication-05-evidence.md"
IMPOSSIBLE = ["R-2", "R-15", "R-16", "R-19", "R-20", "R-22", "R-23", "R-24", "R-28", "R-29", "R-34",
              "R-42", "R-72", "R-82"]
MEASURED = ["K-9a", "K-9b", "K-10", "K-11", "K-12", "K-13", "K-14", "K-15", "K-16", "K-17", "K-19",
            "K-32", "K-33", "K-34", "K-20", "K-21", "K-22", "K-23", "K-24", "K-25"]
# Declared gaps: id -> reason (shown in the table; counted as gaps, never as covered).
DECLARED = {
    "R-36": "deferred to PH-6 (daily digest and reflection; TRACE: 'PH-6 digests'); no spec until its sub-design",
    "R-38": "deferred by ruling (observability database, hub steward); nothing to evidence",
    "R-40": "merged into R-28 (OD-13); evidenced by E-76",
    "R-41": "deferred to the planner (typed assignment/result messages); nothing to evidence yet",
    "R-43": "dropped (same sentence as R-9)",
}


def main() -> int:
    try:
        text = open(DOC, encoding="utf-8").read()
    except OSError as e:
        print(f"TOOLING: {e}", file=sys.stderr)
        return 2
    links: dict[str, set[str]] = {}
    spec_meta: dict[str, dict] = {}
    for m in re.finditer(r"^\*\*(E-\d+) — (.+?)\*\* Links: (.+?)\. Category: (.+?)\. Level: (.+?)\. Tier: (.+?)\. Phase: (.+)$",
                         text, re.M):
        e = m.group(1)
        spec_meta[e] = {"title": m.group(2).rstrip("."), "category": m.group(4), "level": m.group(5),
                        "tier": m.group(6), "phase": m.group(7)}
        for tok in re.split(r",\s*", m.group(3)):
            links.setdefault(tok.strip(), set()).add(e)
    for m in re.finditer(r"^\| (E-\d+) \| ([^|]+) \|", text, re.M):
        e = m.group(1)
        spec_meta.setdefault(e, {"title": "(table row)", "category": "core function", "level": "L7",
                                 "tier": "4", "phase": "in operation"})
        for tok in re.split(r",\s*", m.group(2)):
            links.setdefault(tok.strip(), set()).add(e)
    afc = {}
    for m in re.finditer(r"^\| (AFC-\d+) \| [^|]+ \| [^|]+ \| ([^|]+) \|", text, re.M):
        afc[m.group(1)] = sorted(set(re.findall(r"E-\d+", m.group(2))), key=lambda s: int(s[2:]))
        for e in afc[m.group(1)]:
            links.setdefault(m.group(1), set()).add(e)
    if not spec_meta:
        print("TOOLING: no spec headers parsed", file=sys.stderr)
        return 2
    # every E referenced by AFC must exist
    bad = [e for v in afc.values() for e in v if e not in spec_meta]
    if bad:
        print(f"TOOLING: AFC table names unknown specs {bad}", file=sys.stderr)
        return 2

    def key(e):
        return int(e[2:])

    def row_for(item):
        specs = sorted(links.get(item, set()), key=key)
        # R-n also matches sub-ids like R-67.a1 only when exact; keep exact
        if specs:
            return ", ".join(specs), "covered"
        if item in DECLARED:
            return "—", "GAP (declared): " + DECLARED[item]
        return "—", "GAP (undeclared)"

    out = []
    undeclared = []

    def table(title, items, extra=None):
        out.append(f"\n### {title}\n")
        out.append("| Item | Specs | Status |")
        out.append("|---|---|---|")
        for it in items:
            s, st = row_for(it)
            if extra and it in extra:
                st += f" ({extra[it]})"
            if st == "GAP (undeclared)":
                undeclared.append(it)
            out.append(f"| {it} | {s} | {st} |")

    table("2.1 Requirements R-1 to R-82 (and R-67.a1)", [f"R-{i}" for i in range(1, 83)] + ["R-67.a1"])
    table("2.2 Security invariants SI-1 to SI-31", [f"SI-{i}" for i in range(1, 32)])
    table("2.3 Floor items FX-1 to FX-8", [f"FX-{i}" for i in range(1, 9)])
    # impossible requirements need an adversarial spec
    out.append("\n### 2.4 'Impossible' requirements (REQ field .f): adversarial spec required\n")
    out.append("| Item | Adversarial specs | Status |")
    out.append("|---|---|---|")
    for it in IMPOSSIBLE:
        adv = sorted([e for e in links.get(it, set()) if spec_meta[e]["category"] == "adversarial"], key=key)
        st = "covered" if adv else "GAP (no adversarial spec)"
        if not adv:
            undeclared.append(it + " (adversarial)")
        out.append(f"| {it} | {', '.join(adv) or '—'} | {st} |")
    out.append("\n### 2.5 Every SI-n has an adversarial spec\n")
    out.append("| Item | Adversarial specs | Status |")
    out.append("|---|---|---|")
    for i in range(1, 32):
        it = f"SI-{i}"
        adv = sorted([e for e in links.get(it, set()) if spec_meta[e]["category"] == "adversarial"], key=key)
        st = "covered" if adv else "GAP (no adversarial spec)"
        if not adv:
            undeclared.append(it + " (adversarial)")
        out.append(f"| {it} | {', '.join(adv) or '—'} | {st} |")
    table("2.6 Architecture fail-closed conditions AFC-n (section 13)", sorted(afc, key=lambda s: int(s[4:])))
    table("2.7 Measured properties of ARC4 section 15 (invariant checks, section 10)", MEASURED)
    out.append("\n### 2.8 Specs by category and level\n")
    cats: dict[str, int] = {}
    levs: dict[str, int] = {}
    for e, md in spec_meta.items():
        cats[md["category"]] = cats.get(md["category"], 0) + 1
        for lv in re.findall(r"L[1-7]", md["level"]):
            levs[lv] = levs.get(lv, 0) + 1
    out.append("| Category | Specs |")
    out.append("|---|---|")
    for c in sorted(cats):
        out.append(f"| {c} | {cats[c]} |")
    out.append("")
    out.append("| Level (a spec may name two) | Specs |")
    out.append("|---|---|")
    for lv in sorted(levs):
        out.append(f"| {lv} | {levs[lv]} |")
    out.append(f"\nTotal specs: {len(spec_meta)}. Undeclared gaps: {len(undeclared)}"
               + (f" ({', '.join(undeclared)})" if undeclared else "") + ".")
    print("\n".join(out))
    return 1 if undeclared else 0


if __name__ == "__main__":
    sys.exit(main())
