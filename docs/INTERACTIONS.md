# Interaction Roadmap

`tty-pet watch` is the passive companion mode. The next layer should add small interactions that make the pet feel alive without turning it into an assistant or command runner.

## Recommended Direction

Start with local, harmless interactions that write events into SQLite. The watch view can poll those events and react through mood, phrase, sprite, and bond changes.

This keeps the architecture simple:

```text
tty-pet <interaction> -> SQLite event -> tty-pet watch reacts
```

No sockets, daemon, PTY wrapper, or shell integration are needed.

## First Interaction Commands

### `tty-pet poke`

The simplest direct interaction.

Expected behavior:

- Records a `poke` event for the current project.
- Watch mode briefly switches to a surprised or playful reaction.
- Bond may increase very slightly, with a short cooldown.

Example phrases:

```text
boop?
tiny jump.
hey, paws off.
```

### `tty-pet treat`

A playful reward command.

Expected behavior:

- Records a `treat` event.
- Increases bond more than `poke`.
- Triggers happy/playful mood.
- Should have a cooldown so bond cannot be spammed too easily.

Example phrases:

```text
cronch.
snack acquired.
repo snack accepted.
```

### `tty-pet call`

Gets the pet's attention without implying productivity.

Expected behavior:

- Records a `call` event.
- Watch mode moves the pet toward a visible area or changes phrase.
- Useful when the pet is wandering around a large pane.

Example phrases:

```text
coming.
hop hop.
you rang?
```

### `tty-pet nap`

Sets a calm/sleepy state.

Expected behavior:

- Records a `nap` event.
- Watch mode slows movement briefly.
- Useful when the user wants a quieter pet.

Example phrases:

```text
tiny nap.
soft compile dreams.
zZ
```

## Later Interaction Commands

### `tty-pet name <name>`

Names the project pet.

Recommendation:

- Store the name in `project_pet_state`.
- Keep names project-specific for MVP consistency.
- Display the name in `watch` and `status`.

### `tty-pet mood`

Prints the current resolved mood and why that rule won.

This is more debug-oriented, but it helps contributors understand the rule engine.

### `tty-pet say`

Prints one current phrase without opening the TUI.

This can be useful in scripts, but it should not become shell prompt integration in MVP.

## Watch-Mode Key Interactions

After CLI interactions work, `watch` can support a few direct keys:

```text
p      poke
t      treat
c      call
n      nap
q      quit
ctrl-c quit
```

Key handling should call the same event-recording functions as the CLI commands. Do not duplicate interaction behavior inside the TUI layer.

## Interaction Events

Recommended event kinds:

```text
poke
treat
call
nap
rename
test_pass
test_fail
```

Event handling priority should be:

1. Recent `treat` or `test_pass` -> happy.
2. Recent `poke` -> playful or surprised.
3. Recent `test_fail` -> worried.
4. Recent `nap` -> sleepy.
5. Existing dirty-count and focus rules.

The exact order may change after testing the feel, but explicit user interactions should usually beat passive project state for a short time.

## What Not To Add Yet

Avoid these until the pet feels good with basic interactions:

- Feeding schedules.
- Hunger or death.
- Punishment loops.
- Notifications.
- Auto-running commands.
- AI chat.
- Shell prompt hooks.
- Complex inventory.

The pet should feel easy to keep open, not like another thing the developer has to manage.
