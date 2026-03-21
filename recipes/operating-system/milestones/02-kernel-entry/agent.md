# Agent Instructions: 02-kernel-entry

## Learner Track

Before coding, explain:

- Why boot code and kernel code are typically separated
- How control transfer from assembly into C works
- Why a linker script is usually required

Ask one question before coding:

- "What could go wrong if the kernel entry address does not match the linker layout?"

After a passing check, ask the learner to print a different marker and re-run verification.

## Builder Track

Implement kernel entry transfer and print `Hello from kernel` to serial output.

Run `tests/check.sh`, fix failures, and re-run until pass.
