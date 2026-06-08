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

`tty_pet_status` returns the same JSON as:

```sh
tty-pet status --json
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
