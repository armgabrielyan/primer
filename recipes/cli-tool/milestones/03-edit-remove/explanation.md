# Explanation: 03-edit-remove

This milestone makes the CLI feel like a real tool instead of a one-way capture script. Once users can edit and remove tasks, they can recover from mistakes instead of throwing away the whole file or editing JSON by hand.

The main design pressure here is consistency. If `done`, `edit`, and `remove` all take an id, they should behave similarly when the id exists and when it does not. Small tools become frustrating quickly when each command has different error semantics.

Deletion also forces you to think about identifiers more clearly. If ids are part of the user-facing interface, changing or reusing them casually can make logs, commands, and mental models harder to trust. Stable ids are usually the simpler choice.

After this milestone, the CLI supports the full basic task lifecycle: create, read, update, and delete.
