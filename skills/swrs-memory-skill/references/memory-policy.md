# SWRs Memory Policy v1

## Importance score

`score = 0.35*importance + 0.20*novelty + 0.20*future_utility + 0.15*recurrence + 0.10*user_intent - 0.40*sensitivity_risk`

Save threshold:

- `score >= 0.70`: store in ring buffer.
- `score >= 0.85`: candidate for long-term `MEMORY.md` after replay/verification.
- explicit user memory request can override threshold unless sensitive or unsafe.

## Replay fitness

A trace is promoted when:

- still relevant after time delay
- not duplicated by existing memory
- phrased compactly
- has source/date when useful
- improves future action quality

## Consolidation targets

- User preferences → `MEMORY.md` or `USER.md` if non-sensitive and stable.
- Project/system decisions → `MEMORY.md` or project docs.
- Tooling lessons → `TOOLS.md` or relevant skill trajectory.
- A2A/Search/Emv lessons → matching skill references.

## Safety

- Never store access tokens, passwords, private keys, or full credentials.
- Store that a credential exists only if useful and non-revealing.
- In group/shared contexts, do not expose private long-term memory.
