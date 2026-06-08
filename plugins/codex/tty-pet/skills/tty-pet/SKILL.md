---
name: tty-pet
description: Use tty-pet as a project companion by reading its state and recording safe workflow events.
---

Use the `tty_pet_status` MCP tool to inspect the current project's companion state.

Record only explicit or clearly implied workflow events with `tty_pet_event`:

- `pass` after a meaningful verification step succeeds.
- `fail` after a meaningful verification step fails.
- `poke`, `treat`, `call`, or `nap` only when the user asks for that interaction.

Do not invent events for ordinary file reads or planning. If the user asks to use a custom project directory, set `TTY_PET_PROJECT_DIR` in the environment that launches the MCP server.
