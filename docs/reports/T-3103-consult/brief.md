# External design review: what belongs in a component map's coverage check?

You are an independent reviewer of engineering tooling. Be critical: disagreeing with the recommendation is useful.
Answer in English, at most ~500 words, markdown, with the sections listed at the end. Answer from this text only;
do not run tools.

## Context

A software project keeps a **component fabric**: one YAML "card" per significant file (id, location, purpose,
depends_on, depended_by), used for impact analysis ("what breaks if I change this?") and onboarding. A
`watch-patterns.yaml` file lists globs for the files that *should* have cards. Two automated checks use it:
- **drift:** a watched file with no card is reported as unregistered;
- **coverage (the one firing):** a card whose location is a file that NO watch pattern covers is reported as
  "carded but unwatched". The reasoning: the registry already says that file matters, so the watch list is
  incomplete, and every drift number measures a subset without saying so.

The project's rule for the watch list, written after an earlier incident: **widen only to a natural category boundary
and register every file inside it. Never hand-shape a glob around single files to silence the check.** Hand-shaped
globs were how a previous check got gamed.

## The current warning: 9 carded-but-unwatched locations

1. **8 are BPMN workflow diagrams** in a vendored dependency's folder (`<vendored>/designer/projects/<name>/vN.bpmn`,
   the latest version of each of 8 workflows). They are not source files. A deliberate earlier task "adopted" the 8
   workflows by creating **50 workflow-step cards** (type `workflow-step`), one per step in a diagram. Each card's
   location is the diagram file, with the step id in the card. A separate guard checks these step↔diagram links. The
   folder holds 13 diagram files in all; 5 are older versions with no cards. The vendored tree is replaced wholesale
   when the dependency is re-vendored (about every two months).
2. **1 is a slash-command definition** (`.claude/commands/capture.md`): a markdown file that defines an operator
   command for an AI coding assistant, with a real dependency on a transcript-reader script. 34 such command files
   exist, and only this one has a card.

## Options

- **A (recommended).** Treat the two kinds differently.
  - **Diagrams:** keep the 50 step cards, document why they are exempt, and ask the dependency's maintainers to make
    the coverage check skip cards of type `workflow-step` (they map diagram steps, not source files).
  - **Commands:** cover `.claude/commands/*.md` (the natural boundary) and card all 34 command files, with purposes
    taken from each file's own header.
- **B.** Widen both to natural boundaries: all 13 diagrams (card the 5 old versions too) and all 34 commands.
- **C.** Drop the single `capture.md` card; leave the diagram warning standing.
- **D.** Change nothing.

## What we want from you

1. Is "skip `workflow-step` cards in the coverage check" a principled exemption, or the hand-shaped silencing the rule
   forbids under another name? What test separates the two?
2. For the diagrams, is there a better option we missed? For example: cards pointing at a stable non-vendored location,
   a separate registry for workflow models, or making "latest version only" expressible.
3. Is carding all 34 command files worth it for impact analysis, or is a singleton card plus documentation the honest
   answer?
4. What breaks at the next re-vendor under each option?
5. Your recommendation, with the single most important reason.

Sections: **Verdict** · **Answers (1-4)** · **Risks** · **Recommendation**.
