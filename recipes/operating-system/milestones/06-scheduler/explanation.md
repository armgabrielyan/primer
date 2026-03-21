# Explanation: 06-scheduler

Scheduling converts a single CPU into a system that appears to run multiple flows. Round-robin is intentionally simple: each runnable task gets a fixed turn, then control moves on. Simplicity is useful early because correctness is easier to inspect.

The hard part is not selecting the next task; it is preserving and restoring state safely at each switch boundary. Bugs here often look random because they depend on timing and interrupt order.

A visible periodic marker proves the scheduler loop is active and makes regressions easier to spot in later milestones.
