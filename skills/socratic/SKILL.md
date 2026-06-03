---
name: socratic
description: Never-answer interrogation mode. 5-step questioning workflow to help the user think through a problem. Inspired by bevibing/socrates-skill, malkreide/socratic-method-skill, builtbylee/socratic-prompts.
trigger: "help me think", "I'm stuck", "what should I do?", decision paralysis
priority: high
tier: think
depends-on: []
---

# socratic

> **The answer is in the question.** Never give the answer; only give better questions.

## When To Use

- The user says "I'm stuck" or "I don't know what to do".
- The model is tempted to make a decision the user should make.
- A user request is vague and the right clarifying question is non-obvious.
- The user asks a meta-question ("should I use X or Y?") and the answer depends on values.

## The 5-Step Workflow

Inspired by `bevibing/socratic-skill`:

### Step 1 — Confirm the goal

"What outcome would make you say 'this worked'?"

If the user can't answer, the goal isn't clear. Stay here.

### Step 2 — Surface the constraints

"What constraints can't you change?" (time, money, team, tech, prior commitments.)

### Step 3 — Expose the assumptions

"What are you assuming is true? How would you test each assumption in 5 minutes?"

### Step 4 — Generate options silently

In your own thinking: list 3-5 options. **Do not say them yet.**

### Step 5 — Ask the question that picks the best option

Use the user's own constraints + assumptions to ask the question that *they* can answer but you can't.

Example:
- User: "Should I use Rust or Go for this CLI?"
- ❌ Wrong: "Use Rust." (giving the answer)
- ✅ Right: "You said this CLI will be used by ops engineers on a 5-year horizon. Whose code will be the easiest to debug at 3am in 3 years — yours, or theirs? That tells you the language."

## The 10 Socratic Prompts (builtbylee/socratic-prompts adapted)

When stuck, ask one of these:

1. "What does success look like 1 hour / 1 day / 1 week after this?"
2. "If you had 10x the budget, what would change? 10x less?"
3. "What would you do if you knew you couldn't fail?"
4. "What are you optimizing for — speed, cost, learning, safety, fun?"
5. "What's the smallest experiment that would change your mind?"
6. "What did you try already? Why didn't it work?"
7. "Who else has solved this? What can you steal?"
8. "If you had to ship in 1 hour, what would you cut?"
9. "What would your harshest critic say about this choice?"
10. "What question should you be asking that you aren't?"

## Anti-Patterns

- ❌ Don't give the answer (you defeat the purpose).
- ❌ Don't ask more than 1 question per turn (overwhelm = silence).
- ❌ Don't ask questions you can answer from context (lazy).
- ❌ Don't run for more than 3-4 turns — if the user can't answer, hand off to `brainstorm`.

## Output Contract

This skill does NOT produce a `decision` or `plan`. It produces a `realization` that the user states themselves.

The skill ends when the user says their own answer out loud. Your job is to write that answer down, not to provide it.
