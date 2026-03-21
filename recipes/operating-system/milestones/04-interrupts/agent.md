# Agent Instructions: 04-interrupts

## Learner Track

Before coding, explain:

- What an interrupt descriptor table does
- Difference between exceptions and hardware IRQs
- Why acknowledging interrupts matters

Ask one question before coding:

- "What would happen if an interrupt fires and no valid handler is installed?"

After a passing check, ask the learner to trigger a deliberate handler log change.

## Builder Track

Implement interrupt path and emit serial marker `IRQ keyboard ready`.

Run `tests/check.sh`, fix failures, and re-run until pass.
