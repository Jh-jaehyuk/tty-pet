---
description: Use tty-pet as a project companion by reading its state and recording safe workflow events.
---

Use the `tty_pet_status` MCP tool to inspect the current project's companion state.

MCP tool responses contain structured JSON in `content[0].text`. Prefer `presentation.ko` or `presentation.en` as the user-facing answer, matching the user's language.

When answering from tty-pet data:

- Keep the answer to one or two short sentences.
- Use `state` for factual values such as mood, bond, last test status, and last event.
- Use `reaction` for pet flavor such as phrase and motion.
- Lightly adapt `presentation` wording if needed.
- Do not invent state, give development advice, or turn the pet into a coach.

Record only explicit or clearly implied workflow events with `tty_pet_event`:

- `pass` after a meaningful verification step succeeds.
- `fail` after a meaningful verification step fails.
- `poke`, `treat`, `call`, or `nap` only when the user asks for that interaction.

Do not invent events for ordinary file reads or planning. If the user asks to use a custom project directory, set `TTY_PET_PROJECT_DIR` in the environment that launches the MCP server.
