# Agent Instructions: 03-vga-output

## Learner Track

Before coding, explain:

- Why VGA text mode is memory-mapped I/O
- How character/attribute bytes map to screen cells
- Why serial logging still matters for automated checks

Ask one question before coding:

- "What could happen if VGA memory is written with the wrong address stride?"

After a passing check, ask the learner to change text color and verify behavior.

## Builder Track

Implement VGA text output and emit serial marker `VGA driver ready`.

Run `tests/check.sh`, fix failures, and re-run until pass.
