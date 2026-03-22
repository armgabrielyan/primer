# Agent Instructions: 03-edit-remove

## Learner Track

Before writing code, explain:

- how update and delete operations differ even though both target a task id
- why command consistency matters in small tools
- what can go wrong if error handling is duplicated carelessly

Ask exactly one question before coding:

- "Should a removed task's id be reused later, or is it simpler to keep ids stable forever?"

Then implement together and narrate:

- edit flow
- remove flow
- shared id lookup or validation helpers
- list behavior after mutation

After `tests/check.sh` passes, ask the learner to explain which parts of the command logic now feel duplicated and whether a helper function would improve readability.

## Builder Track

Implement `edit` and `remove` so task lifecycle operations are consistent, scriptable, and satisfy all acceptance criteria in `spec.md`.

Execution rules:

- Run `tests/check.sh`
- If checks fail, fix and re-run until pass
- Keep commentary minimal unless user asks
