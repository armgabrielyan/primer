# Explanation: 07-filesystem

Filesystem milestones introduce trust boundaries with persistent data. Unlike in-memory structures, on-disk bytes can be malformed, stale, or inconsistent. Safe parsing means checking sizes, bounds, and identifiers before using metadata.

A minimal read path is enough to prove architecture direction: map raw blocks to structured records, locate content, and surface deterministic output. The design can remain simple as long as invariants are explicit.

This milestone creates the persistence bridge needed for a useful shell in the next step.
