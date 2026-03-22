# Explanation: 01-bootstrap

This milestone gives you the smallest useful version of a real CLI tool: it accepts commands, persists data, and shows that data back to the user. That is a much more representative beginner project than a script that only prints text.

`argparse` is a good starting point because it gives you structured command parsing, generated help output, and predictable argument validation without extra dependencies. You can grow the command surface one milestone at a time instead of inventing a custom parser.

Using JSON for storage keeps the data model visible. You can open `tasks.json`, inspect it directly, and reason about whether the program state matches the behavior you observed. This makes debugging much easier for new builders.

The core habits in this milestone are:

- turn user input into a command surface
- model data explicitly
- persist state in a human-readable format
- verify behavior by running the CLI, not by assuming it works

Once this passes, you have a concrete foundation for task lifecycle features such as completion, editing, and removal.
