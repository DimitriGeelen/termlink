You are an independent design reviewer. You have no tools and no other context; judge only from the bundle.

The bundle holds a verbatim specification extract for one build unit (an agent key card at a message hub, a post-time signature check against it, and a signed dead letter for retired agent instances), with a map of the current code, and a design note that settles 15 gaps the extract found and splits the unit into two slices.

For EACH of the design note's settlements in its section 3 (items 1 to 15) and for the slicing in its section 2, give one line:
<item>: SOUND | WEAK | WRONG — one sentence why

- SOUND: consistent with the quoted spec and the code map, and closes the gap.
- WEAK: plausible but leaves a hole; say which.
- WRONG: contradicts the quoted spec or the code map, or opens a security hole; say which passage.

Then list at most five findings the note misses entirely (a requirement or evidence item in the extract that neither slice delivers, a security hole, a contradiction), most severe first, each with the quoted passage it rests on. Be strict; do not give credit for intent.
