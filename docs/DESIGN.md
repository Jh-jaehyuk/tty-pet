# tty-pet Design

## Summary

`tty-pet` is a small Rust CLI/TUI application that runs beside a developer's normal terminal session. It shows a cute ASCII pet that reacts to the current project's state.

The core idea is deliberately narrow:

```text
A tiny terminal companion that lives beside your coding session
and reacts to your project state.
```

The project should optimize for installability, charm, and a coherent MVP rather than deep productivity features.

## Decided Direction

- Product type: playful development companion.
- Runtime mode: independent TUI app, not a shell wrapper.
- Observation model: project-state watcher.
- State storage: SQLite.
- Scope: one default pet, optional project-specific image pet, project-specific state, manual test marks, small direct interactions.
- Mood model: rule-based.
- First binary name: `tty-pet`.

## User Experience

The expected use case:

```sh
cd my-project
tty-pet watch
```

The user keeps coding in another terminal pane or tab. The pet reacts to Git dirtiness, manual test events, focus duration, and stored bond.

Small panes are the primary layout target.

```text
my-app  git:3  test:pass
 (=^._.^=)  playful
"tiny paws, big diff."
```

The UI should degrade gracefully in narrow terminals by hiding secondary metadata before hiding the pet.

## Commands

### `tty-pet watch`

Starts the TUI for the current project.

Responsibilities:

- Resolve project identity.
- Create or load project state from SQLite.
- Poll Git dirty count.
- Poll recent events written by other commands.
- Animate the pet.
- Render current mood, phrase, project name, dirty count, test state, and bond.

### `tty-pet pass`

Records a test pass event for the current project.

This command does not need to know whether a test command actually ran. It is intentionally manual for MVP simplicity.

### `tty-pet fail`

Records a test fail event for the current project.

This should not punish the user or reduce bond in a way that makes the toy feel judgmental.

### `tty-pet poke`

Records a `poke` event for the current project. In watch mode, the pet briefly reacts in a playful way.

### `tty-pet treat`

Records a `treat` event for the current project. This increases bond and briefly makes the pet happy.

### `tty-pet call`

Records a `call` event for the current project. This gets the pet's attention without turning the tool into an assistant.

### `tty-pet nap`

Records a `nap` event for the current project. This briefly makes the pet sleepy or quieter.

### `tty-pet image set <path>`

Uses an image file as the current project's pet.

The command validates that the image can be rendered before saving the setting. The original image is not copied into SQLite; SQLite stores the normalized image path and render options.

Supported formats:

- PNG
- JPG
- JPEG

Useful options:

```text
--width <columns>
--height-scale <number>
--charset <dense|simple>
--invert
```

### `tty-pet image clear`

Restores the built-in ASCII pet for the current project.

### `tty-pet image status`

Prints the current project's image pet configuration.

### `tty-pet status`

Prints current project state and debug information.

Useful output:

- Resolved root path.
- Project ID.
- Database path.
- Dirty count if Git is available.
- Current mood.
- Bond.
- Last test status.
- Last event timestamp.

## Project Identity

Project identity should resolve in this order:

1. If inside a Git repository, use the normalized Git root path.
2. Otherwise, use the normalized absolute current directory.

The project ID should be a stable hash of the normalized root path.

Remote URL can be stored as nullable metadata but should not be the primary identity. The same repository cloned to two different directories can have separate pets in the MVP.

## SQLite State

SQLite is the only IPC-like mechanism in MVP.

`tty-pet pass` and `tty-pet fail` write project events. `tty-pet watch` periodically reads recent project events and reacts. No sockets, background daemon, lock server, or shell integration are needed.

Custom image pets are stored as project settings in SQLite. The source image remains on disk and is referenced by path. This keeps the database small and lets users update or replace their own image files intentionally.

Recommended tables are documented in [AGENTS.md](../AGENTS.md).

## Timing

Separate animation frequency from project observation frequency.

```text
animation tick:       150-250ms
event DB polling:     about 2s
state evaluation:     2-5s
git status polling:   5-10s
focus timer updates:  30-60s
```

## Mood Priority

Recent events should feel immediate, so they outrank background project state.

```text
recent fail within 2 minutes  -> worried
recent pass within 90 seconds -> happy
dirty files >= 10             -> busy
dirty files >= 3              -> playful
focus >= 90 minutes           -> sleepy
dirty files == 0              -> calm
otherwise                     -> idle/playful
```

## Pet Content

MVP should ship one default cat-like ASCII pet.

Required sprite states:

- idle
- walk
- happy
- worried
- sleepy
- busy

Phrases are built into code for MVP. A future TOML phrase/skin system can be added after the first usable release.

Phrase principles:

- Short.
- Playful.
- Non-commanding.
- No guilt.
- Light development flavor.

## Custom Image Pet

The image pet path reuses the image-to-ASCII approach from `ascii_image_terminal`, but the renderer lives inside `tty-pet` so the GitHub repository builds independently.

Recommended defaults:

```text
width: 24
height-scale: 0.5
charset: dense
invert: false
```

Watch mode should render a saved image pet when available and fall back to the built-in sprite if the image cannot be opened.

## Storage Locations

Support `TERM_PET_HOME` first. If set, use it for data storage.

Without `TERM_PET_HOME`, use platform-appropriate application directories through a directory helper crate.

`tty-pet status --debug` or equivalent debug output should show the actual database path.

## Out Of Scope

These features should not be implemented in the first release:

- Shell prompt integration.
- PTY wrapper mode.
- Automatic command tracking.
- Automatic test command detection.
- Running user commands.
- Assistant-like suggestions.
- Multiple pets.
- Skin or plugin systems.
- Complex growth systems.
- Productivity scoring.

## Open Questions For Later

- Should `tty-pet run <cmd>` be added after MVP to record exit codes?
- Should project identity optionally merge clones by remote URL?
- Should phrase packs be loaded from config files?
- Should tmux integration be a separate command or just documentation?
- Should bond unlock extra phrases or sprite frames?
