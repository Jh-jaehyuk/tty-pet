# Agent Integration

`tty-pet` can be used by agent clients through two public interfaces:

```sh
tty-pet status --json
tty-pet-mcp
```

`status --json` is the stable scripting interface. `tty-pet-mcp` is the local stdio MCP server for clients such as Claude Code and Codex.

## MCP Server

Run:

```sh
tty-pet-mcp
```

The server reads newline-delimited JSON-RPC messages from stdin and writes JSON-RPC responses to stdout.

Supported methods:

- `initialize`
- `tools/list`
- `tools/call`

Supported tools:

- `tty_pet_status`
- `tty_pet_event`

`tty-pet status --json` returns factual machine JSON for scripts:

```sh
tty-pet status --json
```

`tty_pet_status` and `tty_pet_event` return agent-facing JSON inside MCP text content. The payload includes factual state plus presentation material:

```json
{
  "reaction": {
    "mood": "playful",
    "phrase": "tiny paws, big diff.",
    "motion": "idle",
    "face": [" (=^._.^=)"]
  },
  "state": {
    "mood": "playful",
    "bond": 7,
    "last_test_status": "pass",
    "last_event_kind": "treat"
  },
  "presentation": {
    "ko": "현재 펫은 playful 상태예요. 방금 treat 이벤트를 기억하고 짧게 반응하고 있습니다.",
    "en": "The pet is currently playful. It still remembers the recent treat and is giving a small reaction.",
    "markdown": "```\n (=^._.^=)\n```\n현재 펫은 playful 상태예요. 방금 treat 이벤트를 기억하고 짧게 반응하고 있습니다.",
    "style": "short_playful",
    "rules": [
      "Do not invent state.",
      "Do not give development advice.",
      "Keep it to one or two short sentences.",
      "When the user wants to see the pet, include the face from presentation.markdown."
    ]
  }
}
```

`tty_pet_event` accepts:

```json
{
  "kind": "poke"
}
```

Allowed `kind` values:

```text
poke
treat
call
nap
pass
fail
```

The MCP server does not expose arbitrary shell execution.

Agent clients should treat `presentation.ko` and `presentation.en` as ready-to-use text. When the user wants to see the pet inside the agent conversation, use `presentation.markdown` so the ASCII face appears with the short response. Agents may lightly adapt wording to match the user's language, but should not invent state, add development advice, or expand the answer beyond one or two short sentences.

## Project Resolution

By default, `tty-pet` resolves the project from the current working directory.

Agent clients can set this environment variable when the MCP server is launched:

```sh
TTY_PET_PROJECT_DIR=/path/to/project
```

This is useful when a plugin starts the MCP process from a plugin directory rather than the active project root.

## Claude Code

The repository includes a Claude Code plugin bundle at:

```text
plugins/claude-code/tty-pet
```

The bundle contains:

- `.claude-plugin/plugin.json`
- `.mcp.json`
- `skills/tty-pet/SKILL.md`

Use a local stdio MCP configuration that launches `tty-pet-mcp`:

```json
{
  "mcpServers": {
    "tty-pet": {
      "type": "stdio",
      "command": "tty-pet-mcp",
      "env": {
        "TTY_PET_PROJECT_DIR": "${CLAUDE_PROJECT_DIR:-.}"
      }
    }
  }
}
```

Claude Code plugins can package MCP server configurations, skills, and hooks. The bundled first plugin only packages this MCP configuration and a short skill guide. Avoid automatic pass/fail hooks until test-result detection is explicit.

## Codex

The repository includes a Codex plugin bundle at:

```text
plugins/codex/tty-pet
```

The bundle contains:

- `.codex-plugin/plugin.json`
- `.mcp.json`
- `skills/tty-pet/SKILL.md`

Use the same MCP boundary:

```json
{
  "mcpServers": {
    "tty-pet": {
      "command": "tty-pet-mcp"
    }
  }
}
```

For a Codex plugin, keep the plugin thin:

- Add `.codex-plugin/plugin.json`.
- Add `.mcp.json` pointing at `tty-pet-mcp`.
- Add a skill explaining when to call `tty_pet_status` and `tty_pet_event`.

Do not bundle native binaries in the first plugin version. Ask users to install `tty-pet` first:

```sh
cargo install --git https://github.com/Jh-jaehyuk/tty-pet --locked
```

## Future Plugin Work

Recommended next steps:

1. Add a `tty_pet_image_set` MCP tool if image selection through agents becomes useful.
2. Add plugin validation commands to release docs.
3. Consider hooks only for playful events like `call` or `nap`, not test pass/fail.
