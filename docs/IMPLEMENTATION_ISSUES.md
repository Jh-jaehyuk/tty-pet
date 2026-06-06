# Implementation Issues

This is the first implementation backlog for `tty-pet`. It is written as issue-ready work, but no issue tracker has been initialized yet.

## TP-001: Create Rust CLI Scaffold

Set up the initial Rust project only when scaffolding is requested.

Acceptance criteria:

- Binary name is `tty-pet`.
- CLI has subcommands: `watch`, `pass`, `fail`, `status`.
- Commands dispatch to placeholder handlers.
- No TUI or SQLite behavior is required yet.

Suggested crates:

- `clap`
- `anyhow` or `thiserror`

## TP-002: Resolve App Data Directory

Implement data path resolution.

Acceptance criteria:

- `TERM_PET_HOME` overrides all other data locations.
- Default path uses platform application data directory.
- Database path is exposed to command handlers.
- Tests can isolate data paths with `TERM_PET_HOME`.

Suggested crates:

- `directories`
- `tempfile` for tests

## TP-003: Resolve Project Identity

Implement current project resolution.

Acceptance criteria:

- Git root is used when inside a Git repository.
- Normalized current directory is used outside Git.
- Project ID is a stable hash of normalized root path.
- Nullable Git remote URL metadata is read when available.
- Missing Git CLI is handled gracefully.

## TP-004: Add SQLite Schema And Migrations

Create database setup and migrations.

Acceptance criteria:

- SQLite database is created on first use.
- Tables exist for projects, project pet state, and project events.
- Migrations are idempotent.
- Database code is isolated from TUI rendering code.

Suggested crate:

- `rusqlite`

## TP-005: Implement Project Repository Operations

Add database read/write functions for project state.

Acceptance criteria:

- Ensure project row exists.
- Ensure project pet state row exists.
- Insert project event.
- Read latest project state.
- Read latest event for a project.
- Update last seen timestamp.

## TP-006: Implement `tty-pet pass`

Record test pass events.

Acceptance criteria:

- Resolves current project.
- Inserts `test_pass` event.
- Updates last test status.
- Increments bond by a small amount.
- Prints a short confirmation.

## TP-007: Implement `tty-pet fail`

Record test fail events.

Acceptance criteria:

- Resolves current project.
- Inserts `test_fail` event.
- Updates last test status.
- Does not reduce bond in MVP.
- Prints a short confirmation.

## TP-008: Implement Git Dirty Count Polling

Read Git dirty file count.

Acceptance criteria:

- Uses `git status --porcelain`.
- Counts changed lines as dirty entries.
- Returns zero for clean repos.
- Returns an unavailable state outside Git or when Git fails.
- Does not panic when Git is missing.

## TP-009: Implement Rule-Based Mood Engine

Convert project state into mood.

Acceptance criteria:

- Recent fail within 120 seconds returns `worried`.
- Recent pass within 90 seconds returns `happy`.
- Dirty files >= 10 returns `busy`.
- Dirty files >= 3 returns `playful`.
- Focus >= 90 minutes returns `sleepy` when higher-priority rules do not match.
- Clean repo returns `calm`.
- Fallback returns `idle` or `playful`.

## TP-010: Add Built-In Sprites And Phrases

Define the default pet content.

Acceptance criteria:

- Default pet has idle, walk, happy, worried, sleepy, and busy sprites.
- Phrases are grouped by mood/event.
- Phrases are short enough for narrow panes.
- No external phrase config is required.

## TP-011: Build TUI Terminal Lifecycle

Set up `ratatui` and `crossterm`.

Acceptance criteria:

- `tty-pet watch` enters alternate screen or a clear terminal mode according to the chosen implementation.
- Terminal cleanup runs on normal exit.
- Terminal cleanup runs on error paths where practical.
- Ctrl-C or `q` exits cleanly.

Suggested crates:

- `ratatui`
- `crossterm`

## TP-012: Implement Watch Loop

Build the main watch loop.

Acceptance criteria:

- Animation ticks every 150-250ms.
- Recent events are polled from SQLite about every 2s.
- Git dirty count is polled every 5-10s.
- Focus duration is updated periodically.
- Mood is recalculated from the latest observed state.

## TP-013: Render Responsive Small-Pane Layout

Implement the watch view.

Acceptance criteria:

- Layout works at roughly 32x8.
- Pet remains visible in narrow panes.
- Secondary metadata is hidden before the sprite is hidden.
- Long phrases are truncated or wrapped safely.
- Rendering does not panic on very small terminal sizes.

## TP-014: Implement `tty-pet status`

Print current project status.

Acceptance criteria:

- Shows resolved root path.
- Shows project ID.
- Shows database path.
- Shows bond, mood, last test status, and last event.
- Shows dirty count when available.
- Output is readable in plain terminals.

## TP-015: Add First Integration Tests

Cover core behavior before polishing.

Acceptance criteria:

- Project identity tests cover Git and non-Git directories.
- Mood priority tests cover pass/fail over dirty count.
- Database tests run against a temp data path.
- Pass/fail command tests update only the current project.

## TP-016: Add README Usage Examples And Release Notes

Update documentation after the scaffold exists.

Acceptance criteria:

- README install instructions match the actual package/release path.
- README command examples work.
- Non-goals remain explicit.
- First release notes explain manual `pass`/`fail` behavior.

## TP-017: Add Basic Interaction Events

Add direct toy-like interaction commands.

Acceptance criteria:

- [x] `tty-pet poke` records a `poke` event for the current project.
- [x] `tty-pet treat` records a `treat` event for the current project.
- [x] `tty-pet call` records a `call` event for the current project.
- [x] `tty-pet nap` records a `nap` event for the current project.
- [x] Interaction commands use the same project resolution and database path behavior as `pass` and `fail`.
- [x] Watch mode reacts to recent interaction events through mood and phrase changes.

## TP-018: Add Watch-Mode Key Interactions

Let users interact with the pet while the TUI is open.

Acceptance criteria:

- [x] `p` records a `poke` event.
- [x] `t` records a `treat` event.
- [x] `c` records a `call` event.
- [x] `n` records a `nap` event.
- [x] `q` and Ctrl-C still exit cleanly.
- [x] Key interactions share the same event-writing path as CLI interactions.

## TP-019: Add Project-Specific Pet Name

Let users name the project pet.

Acceptance criteria:

- `tty-pet name <name>` stores a project-specific name.
- `tty-pet status` shows the pet name.
- `tty-pet watch` shows the pet name when space allows.
- Names are stored in SQLite and do not require config files.

## TP-020: Add Project-Specific Image Pet

Use a chosen image file as the current project's pet.

Acceptance criteria:

- [x] `tty-pet image set <path>` validates and saves a PNG/JPG/JPEG image pet.
- [x] `tty-pet image clear` restores the built-in pet.
- [x] `tty-pet image status` shows saved image settings.
- [x] Image settings persist in SQLite across runs.
- [x] Watch mode renders the custom ASCII image when configured.
- [x] Watch mode falls back to the built-in pet when the image cannot be rendered.
- [x] The project remains independently buildable without a local `ascii_image_terminal` dependency.

## Deferred Issues

These are intentionally not part of MVP:

- `tty-pet run <cmd>` command wrapper.
- Shell prompt integration.
- tmux status integration.
- TOML phrase packs.
- Multiple pets or skins.
- Remote URL based project merging.
- Background daemon or IPC.
