# Explanation: 02-kernel-entry

This milestone is the first controlled handoff from firmware-loaded boot code into your kernel. Boot assembly runs in a constrained environment and is good at setup and jumps; C is better for maintainable kernel logic. The handoff point must match your linker layout, symbol names, and calling assumptions.

A common failure mode is jumping to the wrong address: the CPU executes garbage and the VM resets or hangs. Another is stack or segment setup mismatches before entering C. That is why deterministic build/link settings and a stable entry symbol matter early.

Once this works, your project is no longer just a boot sector. You now have the foundation of a real kernel execution path.
