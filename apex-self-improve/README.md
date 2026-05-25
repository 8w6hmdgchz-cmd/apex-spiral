# APEX Self-Improvement Loop

Purpose: run a bounded self-improvement loop every 15 minutes using APEX-style formula analysis.

User requested sequence:

1. First 5 foundational rounds alternate order: `21354 -> 12534`.
2. After 5 foundational rounds, switch to alternating `12354` and `21354`.
3. Each round:
   - Substitute current assistant state into formula.
   - Find formula/process bugs.
   - Repair the bug in prompt/skill/docs/task process where safe.
   - Re-run the corrected formula on self.
   - Verify improvement with evidence.
   - Include biology/chemistry/physics formula learning and mapping.
   - Use public open-source references only when useful; avoid risky/irrelevant sources.

Safety bounds:

- No external writes, posts, messages, account actions, or trading actions.
- No downloading/running unknown code.
- Web/GitHub lookups are read-only and source-grounded.
- Output must separate fact / inference / hypothesis / next verification.
- If no real improvement is verifiable, say so and reduce confidence.

Canonical step meanings:

1. Substitute self into formula.
2. Find formula/process bug.
3. Repair the bug.
4. Re-substitute with corrected formula and learn.
5. Verify improvement.

State file: `apex-self-improve/state.json`.
Logs: `apex-self-improve/logs/`.


## Core Principles

See `apex-self-improve/principles.md`. Every loop must treat formulas as mirrors, bugs as tests, and tool-use as awakening.
