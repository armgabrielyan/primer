# Explanation: 04-interrupts

Interrupts turn your kernel from passive code into reactive code. Hardware signals are mapped through controller logic into IRQ lines, which your kernel routes via descriptor tables to handlers. If mapping, masking, or acknowledgements are wrong, you get hangs, repeated interrupts, or silent device failures.

This milestone is also about control boundaries. Interrupt handlers should do minimal work, record state, and hand off heavier processing safely. That separation is critical once scheduling and memory management become more complex.

A stable keyboard IRQ path is a practical checkpoint that your interrupt plumbing is functional.
