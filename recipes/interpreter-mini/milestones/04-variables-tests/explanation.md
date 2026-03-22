# Explanation: 04-variables-tests

Variables turn isolated expressions into programs with state. Once one statement can define a value that later statements depend on, the interpreter needs an environment that survives across execution steps.

This milestone also changes the execution model slightly. Instead of evaluating a single expression string, the interpreter now reads a file, processes one statement at a time, updates the environment, and prints the result of the final expression statement. That makes the language feel much more like a real tool.

Tests become especially valuable here because the final feature relies on every earlier stage:

- tokenization
- parsing
- evaluation
- environment handling

If any of those regress, the whole language behavior can break. A small test suite gives you a durable way to catch that.

By the end of this milestone, you have a complete small interpreter pipeline rather than disconnected demos.
