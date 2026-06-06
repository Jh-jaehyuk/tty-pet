# Agent Guide

This repository is for `tty-pet`, a Rust CLI/TUI project. The scaffold exists; keep changes scoped to the documented MVP and commit meaningful feature/documentation increments.

## Product Contract

`tty-pet` is a cute terminal toy that reacts to a developer's current project. It is not an assistant, shell replacement, command runner, or productivity monitor.

Keep the MVP small:

- `tty-pet watch`
- `tty-pet pass`
- `tty-pet fail`
- `tty-pet poke`
- `tty-pet treat`
- `tty-pet call`
- `tty-pet nap`
- `tty-pet image set/clear/status`
- `tty-pet status`
- One default cat-like ASCII pet plus optional project-specific image pet
- Per-project SQLite state
- Rule-based mood selection
- Git dirty count polling

Avoid expanding the first release into shell integration, PTY wrapping, automatic command detection, plugins, themes, or multiple pets.

## Architecture Shape

Prefer this module structure:

```text
src/
  main.rs
  app.rs
  cli.rs
  config.rs
  db/
    mod.rs
    migrations.rs
    models.rs
    repository.rs
  project/
    mod.rs
    git.rs
    identity.rs
  mood/
    mod.rs
    rules.rs
    phrases.rs
  pet/
    mod.rs
    built_in.rs
    custom_image.rs
  tui/
    mod.rs
    app_state.rs
    render.rs
    terminal.rs
  time.rs
  error.rs
```

Expected responsibilities:

- `cli`: Parse commands and dispatch top-level flows.
- `config`: Resolve `TERM_PET_HOME`, platform data directories, and debug paths.
- `db`: Own SQLite connection setup, migrations, row models, and read/write operations.
- `project`: Resolve project identity from Git root or normalized current directory.
- `project::git`: Shell out to `git status --porcelain` and parse only the dirty count.
- `mood`: Convert observed state and recent events into a small mood enum.
- `mood::phrases`: Store built-in phrase tables for the MVP.
- `pet`: Own built-in sprites and image-to-ASCII custom pet rendering.
- `tui`: Own terminal lifecycle, tick loop, rendering, and responsive layout.
- `app`: Coordinate project observation, DB reads/writes, and view state.

Do not let rendering code query Git or write to SQLite directly. Keep side effects in the app/database/project layers.

## Core Data Model

Recommended tables:

```text
projects
  id text primary key
  root_path text not null unique
  git_remote_url text null
  created_at text not null
  last_seen_at text not null

project_pet_state
  project_id text primary key references projects(id)
  bond integer not null default 0
  mood text not null default 'idle'
  last_test_status text null
  last_event_kind text null
  last_event_at text null
  focus_started_at text null
  custom_image_path text null
  custom_image_width integer null
  custom_image_height_scale real null
  custom_image_charset text null
  custom_image_invert integer null
  updated_at text not null

project_events
  id integer primary key autoincrement
  project_id text not null references projects(id)
  kind text not null
  created_at text not null
```

Project IDs should be stable hashes of normalized root paths. Store the full `root_path` separately for display and debugging. Store `git_remote_url` only as nullable metadata.

## Mood Rules

Use a simple ordered rule list:

1. Recent `test_fail` within 120 seconds -> `worried`.
2. Recent `test_pass` within 90 seconds -> `happy`.
3. Dirty files >= 10 -> `busy`.
4. Dirty files >= 3 -> `playful`.
5. Focus time >= 90 minutes -> `sleepy`.
6. Dirty files == 0 -> `calm`.
7. Fallback -> `idle` or `playful`.

Do not introduce scoring, machine learning, or complex state machines for the MVP.

## Event Flow

`tty-pet watch`:

1. Resolve the current project.
2. Ensure database and project rows exist.
3. Enter TUI loop.
4. Tick animation every 150-250ms.
5. Refresh project state every 2-5s.
6. Poll Git dirty count every 5-10s.
7. Re-read recent SQLite events every 2s.
8. Render mood, sprite, phrase, dirty count, last test status, and bond.

`tty-pet pass`:

1. Resolve the current project.
2. Insert `test_pass` event.
3. Update last test state and small bond increment.

`tty-pet fail`:

1. Resolve the current project.
2. Insert `test_fail` event.
3. Update last test state without punitive counters.

`tty-pet poke/treat/call/nap`:

1. Resolve the current project.
2. Insert the matching interaction event.
3. Apply any small bond change from the shared interaction spec.
4. Let `watch` react by polling recent SQLite events.

`tty-pet image set/clear/status`:

1. Resolve the current project.
2. Validate the image can render before saving it.
3. Store or clear image path and render options in SQLite.
4. Let `watch` render the configured image pet, falling back to the built-in sprite on error.

`tty-pet status`:

1. Resolve the current project.
2. Print root path, project ID, database path, dirty count if available, bond, mood, and last event.

## Implementation Issues

Use the implementation issue list in [docs/IMPLEMENTATION_ISSUES.md](docs/IMPLEMENTATION_ISSUES.md) as the working backlog. Each issue should remain independently shippable and avoid pulling in non-MVP scope.

## Engineering Rules

- Prefer small, boring Rust modules over clever abstractions.
- Keep terminal rendering deterministic enough to test.
- Avoid direct sleeps in testable logic; inject clocks where practical.
- Keep database migrations explicit and idempotent.
- Make `TERM_PET_HOME` work early so tests do not write into a real user data directory.
- Use `git` CLI calls for MVP Git status instead of `git2`.
- Treat missing Git, non-Git directories, and Git command failures as normal states.
- Keep all pet phrases short enough for narrow terminal panes.
- Store custom image settings in SQLite, not copied image blobs.
- Keep `tty-pet` independently buildable; do not depend on a local checkout of `ascii_image_terminal`.

## Testing Priorities

High-value tests for the first scaffold:

- Project identity resolves Git root before current directory.
- Non-Git directories get stable normalized path identities.
- Mood priority handles recent pass/fail over dirty count.
- `TERM_PET_HOME` controls database location.
- DB migrations create all expected tables.
- `pass` and `fail` commands update only the current project.
- TUI layout handles narrow widths without panics.
- Image pet settings can be set and cleared.
- Custom image renderer maps dark pixels to dense ASCII characters.
