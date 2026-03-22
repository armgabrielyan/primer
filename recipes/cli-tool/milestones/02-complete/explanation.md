# Explanation: 02-complete

The moment your tool supports updates, task identity matters. A task should be referred to by something stable even if the order in the file changes later. That is why this milestone makes integer ids part of the public CLI behavior instead of treating list position as an implementation detail.

Marking a task as done is a small feature, but it introduces an important pattern: read state, locate one record, update just the relevant field, and write the full state back safely. That loop appears in many real tools and services.

This milestone also adds error handling as part of the contract. A user asking to complete task `99` should not get a silent no-op. Good CLI tools explain what failed and signal failure with a non-zero exit code so the command is scriptable.

After this milestone, your tool has the core idea of a task lifecycle rather than a write-only list.
