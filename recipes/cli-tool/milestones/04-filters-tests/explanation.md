# Explanation: 04-filters-tests

Filtering and summaries make a CLI feel usable over time. Once a task list grows, users need quick ways to answer practical questions such as "what is still open?" and "how much have I finished?" These are small features with high product value.

This milestone also introduces tests as part of the finished deliverable. That matters because command-line tools are especially easy to regress with small argument-parsing or formatting changes. A lightweight test suite gives you a repeatable way to protect behavior as the project evolves.

There are multiple valid testing strategies here:

- test the CLI as a subprocess and assert on output
- extract storage or formatting helpers and test them directly
- combine both approaches

The important thing is that the tests cover real behavior and remain easy to run. By the end of this milestone, the project should feel like a complete small tool rather than a tutorial fragment.
