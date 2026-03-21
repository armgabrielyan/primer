# Agent Instructions: 06-scheduler

## Learner Track

Before coding, explain:

- Why preemption needs timer-driven events
- Round-robin fairness and its limits
- Why scheduler state transitions must be observable

Ask one question before coding:

- "What failure would you expect if context state is only partially saved?"

After a passing check, ask the learner to reason about starvation in non-round-robin schedulers.

## Builder Track

Implement round-robin scheduling and emit serial marker `Scheduler tick`.

Run `tests/check.sh`, fix failures, and re-run until pass.
