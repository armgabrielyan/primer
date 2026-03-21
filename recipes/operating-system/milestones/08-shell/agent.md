# Agent Instructions: 08-shell

## Learner Track

Before coding, explain:

- Why shells separate parsing from execution
- How command loops handle invalid input safely
- Which subsystems from earlier milestones the shell depends on

Ask one question before coding:

- "How would you extend a minimal shell without tightly coupling parsing and execution?"

After a passing check, ask the learner to add one extra built-in command.

## Builder Track

Implement shell loop and emit prompt marker `shell>`.

Run `tests/check.sh`, fix failures, and re-run until pass.
