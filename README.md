# tty-pet

`tty-pet` is a tiny terminal companion that lives beside your coding session and reacts to your project state.

It is intentionally more toy than productivity tool: a cute, playful TUI pet that watches a project, reacts to Git dirtiness and manually marked test results, and remembers a small per-project bond in SQLite. You can use the built-in ASCII pet or set a project-specific pet from an image file.

## Installation

Install from GitHub:

```sh
cargo install --git https://github.com/Jh-jaehyuk/tty-pet --locked
```

Install from a local checkout while developing:

```sh
git clone https://github.com/Jh-jaehyuk/tty-pet.git
cd tty-pet
cargo install --path . --locked
```

After pulling updates from this repository, reinstall the local binary:

```sh
git pull
cargo install --path . --locked
```

Check that both binaries are available:

```sh
tty-pet status --json
tty-pet-mcp
```

## Usage

Start the pet in the project you want it to watch:

```sh
cd my-project
tty-pet watch
```

Use another terminal tab, tmux pane, or agent client to send events:

```sh
tty-pet pass
tty-pet fail
tty-pet poke
tty-pet treat
tty-pet call
tty-pet nap
```

Set a project-specific image pet:

```sh
tty-pet image set ~/Pictures/pet.png
tty-pet image clear
tty-pet image status
```

Inspect project state:

```sh
tty-pet status
tty-pet status --json
```

`tty-pet watch` opens a small responsive TUI. It is meant to run in a second terminal tab, a tmux split, or a side pane while the developer keeps using their normal shell.

`tty-pet pass` and `tty-pet fail` record test-result events for the current project. The watch view polls SQLite and reacts shortly after.

`tty-pet poke`, `tty-pet treat`, `tty-pet call`, and `tty-pet nap` record small interaction events for the current project. In watch mode, the same interactions are available as `p`, `t`, `c`, and `n`.

`tty-pet image set <path>` stores a project-specific image pet. PNG, JPG, and JPEG files are rendered as ASCII and used by `watch` on the next refresh and future runs.

`tty-pet status` prints the current project's stored pet state and debug information such as the resolved project root and database path.

`tty-pet status --json` prints the same project state as machine-readable JSON for agents and scripts.

`tty-pet-mcp` starts a local stdio MCP server that exposes safe `tty-pet` tools to agent clients.

## MVP Scope

The first release is scoped to one default pet, passive watch mode, small direct interactions, custom image pets, agent-facing MCP tools, and project-local state.

## Agent Plugins

`tty-pet` includes thin plugin bundles for agent clients:

- `plugins/codex/tty-pet`
- `plugins/claude-code/tty-pet`

Install the CLI first so the plugin MCP configuration can find `tty-pet-mcp` on `PATH`:

```sh
cargo install --git https://github.com/Jh-jaehyuk/tty-pet --locked
```

Both plugin bundles expose the same local MCP tools:

- `tty_pet_status`
- `tty_pet_event`

Claude Code can load the bundled plugin from a local checkout:

```sh
cd ~/tty-pet
claude --plugin-dir ./plugins/claude-code/tty-pet
```

Then ask Claude Code for interactions such as:

```text
tty-pet 상태와 얼굴을 보여줘.
treat 줘.
테스트가 통과했으니 pass 이벤트 기록해줘.
```

MCP responses include structured state, short Korean/English presentation text, and an ASCII face through `presentation.markdown`, so agent replies can show the pet inside the conversation.

See [Agent Integration](docs/AGENT_INTEGRATION.md) for setup notes and client-specific details.

## What It Watches

The MVP observes project state, not the user's shell session.

- Current project identity, resolved from Git root first and normalized current directory second.
- Git dirty file count using `git status --porcelain`.
- Last manually recorded test event.
- Focus/session duration while `watch` is running.
- Per-project bond and mood stored in SQLite.
- Per-project custom image pet settings.

Shell integration, command interception, PTY wrapping, tmux automation, and automatic test command detection are deliberately out of scope for the first release.

## Product Feel

`tty-pet` should feel like a cute terminal toy that happens to know a little about the repo.

The pet should:

- Move and react in a small terminal pane.
- Speak in short playful lines.
- Avoid guilt, nagging, productivity scoring, and command execution.
- Prefer charm over correctness when the two conflict.
- Stay quiet enough to be left open for long coding sessions.

Example reactions:

```text
tiny paws, big diff.
green feels crunchy.
uh oh.
this repo smells like snacks.
```

## Mood Rules

Mood selection is rule-based. Recent events beat project state, and project state beats time-based idle behavior.

```text
recent fail within 2 minutes  -> worried
recent pass within 90 seconds -> happy
dirty files >= 10             -> busy
dirty files >= 3              -> playful
focus >= 90 minutes           -> sleepy
dirty files == 0              -> calm
otherwise                     -> idle/playful
```

## Planned Stack

- Language: Rust
- CLI: `clap`
- TUI: `ratatui` + `crossterm`
- Database: `rusqlite`
- Directories: `directories`
- Image rendering: `image`
- Git state: shell out to `git status --porcelain` for the MVP

## Data Location

`tty-pet` should use platform-appropriate application directories, with an explicit override for tests and power users.

```text
TERM_PET_HOME=/custom/path
```

If `TERM_PET_HOME` is set, the SQLite database lives at:

```text
$TERM_PET_HOME/tty-pet.db
```

Otherwise the app should use the platform data directory returned by the selected directory helper crate.

## Non-Goals For The First Release

- Shell prompt integration
- PTY wrapper mode
- Command execution suggestions
- Automatic test command detection
- Multiple pets or skins
- Phrase/config file loading
- Automatic agent hooks that infer pass/fail without explicit user or agent intent
- Growth systems beyond a small per-project bond counter

## Documentation

- [Design](docs/DESIGN.md)
- [Agent Integration](docs/AGENT_INTEGRATION.md)
- [Interaction Roadmap](docs/INTERACTIONS.md)
- [Implementation Issues](docs/IMPLEMENTATION_ISSUES.md)
- [Agent Guide](AGENTS.md)
