# Explanation: 03-vga-output

VGA text mode gives you deterministic screen output with no graphics stack. You write directly to a fixed memory region where each cell is two bytes: one for the character and one for style attributes. This is often the first hardware-facing driver in hobby OS flows.

Even with VGA working, serial output remains important. CI and headless checks cannot reliably inspect the rendered frame buffer, so a serial marker is the stable verification channel.

This milestone establishes a pattern used later: user-visible behavior through one interface and machine-verifiable behavior through another.
