# Explanation: 05-memory

Memory management is where kernel reliability usually starts to diverge between demos and systems that can grow. Early allocators are intentionally simple, but they still need strict invariants: no overlapping allocations, deterministic free behavior, and predictable startup state.

Even before full virtual memory, frame-level allocation discipline matters because every later subsystem depends on it, including stacks, buffers, and process metadata. Subtle allocator corruption often appears far from the bug site.

This milestone establishes a small but trustworthy allocation core that later milestones can rely on.
