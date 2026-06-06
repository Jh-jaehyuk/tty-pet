use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;

use crate::app::AppContext;
use crate::db::{self, repository};
use crate::interactions::{self, Interaction};
use crate::mood::phrases;
use crate::mood::rules::{self, ObservedState};
use crate::pet::custom_image;
use crate::project::git;
use crate::time;
use crate::tui::app_state::WatchState;
use crate::tui::render;

pub fn run(context: AppContext) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let result = run_loop(&mut terminal, context);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: Backend>(terminal: &mut Terminal<B>, context: AppContext) -> Result<()> {
    let connection = db::open(&context.paths.db_path)?;
    let mut state = WatchState::new(context.project.display_name());
    let mut last_animation = Instant::now();
    let mut last_refresh = Instant::now() - Duration::from_secs(10);
    let mut last_git_poll = Instant::now() - Duration::from_secs(10);
    let mut dirty_count = git::dirty_count(&context.project.root_path)?;

    loop {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key_action(key.code, key.modifiers) {
                        KeyAction::Quit => break,
                        KeyAction::Interact(interaction) => {
                            interactions::record(&connection, &context.project.id, interaction)?;
                            last_refresh = Instant::now() - Duration::from_secs(2);
                        }
                        KeyAction::Ignore => {}
                    }
                }
            }
        }

        if last_git_poll.elapsed() >= Duration::from_secs(5) {
            dirty_count = git::dirty_count(&context.project.root_path)?;
            last_git_poll = Instant::now();
        }

        if last_refresh.elapsed() >= Duration::from_secs(2) {
            let pet_state = repository::pet_state(&connection, &context.project.id)?;
            let latest_event = repository::latest_event(&connection, &context.project.id)?;
            let now = time::now_unix_seconds();
            let event_age = latest_event
                .as_ref()
                .and_then(|event| event.created_at.parse::<i64>().ok())
                .map(|created_at| now.saturating_sub(created_at) as u64);
            let latest_event_kind = latest_event.map(|event| event.kind);
            let observed = ObservedState {
                recent_event_kind: latest_event_kind.clone(),
                recent_event_age_secs: event_age,
                dirty_count,
                focus_minutes: state.focus_minutes(),
            };
            let mood = rules::evaluate(&observed);

            repository::update_mood(&connection, &context.project.id, mood.as_str())?;

            state.mood = mood;
            state.dirty_count = dirty_count;
            state.bond = pet_state.bond;
            state.last_test_status = pet_state.last_test_status;
            update_custom_sprite(&mut state, pet_state.custom_image.as_ref());
            state.phrase =
                phrases::phrase_for_event(mood, latest_event_kind.as_deref(), state.frame)
                    .to_string();
            last_refresh = Instant::now();
        }

        if last_animation.elapsed() >= Duration::from_millis(200) {
            state.advance_frame();
            terminal.draw(|frame| render::render(frame, &state))?;
            last_animation = Instant::now();
        }
    }

    Ok(())
}

fn update_custom_sprite(
    state: &mut WatchState,
    config: Option<&crate::db::models::CustomImageConfig>,
) {
    let Some(config) = config else {
        state.custom_sprite = None;
        state.custom_sprite_key = None;
        return;
    };
    let key = config.render_key();

    if state.custom_sprite_key.as_deref() == Some(key.as_str()) {
        return;
    }

    match custom_image::render_config(config) {
        Ok(rendered) => {
            state.custom_sprite = Some(rendered.lines);
            state.custom_sprite_key = Some(key);
        }
        Err(_) => {
            state.custom_sprite = None;
            state.custom_sprite_key = None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    Quit,
    Interact(Interaction),
    Ignore,
}

fn key_action(code: KeyCode, modifiers: KeyModifiers) -> KeyAction {
    match code {
        KeyCode::Char('q') => KeyAction::Quit,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => KeyAction::Quit,
        KeyCode::Char('p') => KeyAction::Interact(Interaction::Poke),
        KeyCode::Char('t') => KeyAction::Interact(Interaction::Treat),
        KeyCode::Char('c') => KeyAction::Interact(Interaction::Call),
        KeyCode::Char('n') => KeyAction::Interact(Interaction::Nap),
        _ => KeyAction::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_watch_interaction_keys() {
        assert_eq!(
            key_action(KeyCode::Char('p'), KeyModifiers::empty()),
            KeyAction::Interact(Interaction::Poke)
        );
        assert_eq!(
            key_action(KeyCode::Char('t'), KeyModifiers::empty()),
            KeyAction::Interact(Interaction::Treat)
        );
        assert_eq!(
            key_action(KeyCode::Char('c'), KeyModifiers::empty()),
            KeyAction::Interact(Interaction::Call)
        );
        assert_eq!(
            key_action(KeyCode::Char('n'), KeyModifiers::empty()),
            KeyAction::Interact(Interaction::Nap)
        );
    }

    #[test]
    fn ctrl_c_quits_instead_of_calling_pet() {
        assert_eq!(
            key_action(KeyCode::Char('c'), KeyModifiers::CONTROL),
            KeyAction::Quit
        );
    }

    #[test]
    fn q_quits_watch_mode() {
        assert_eq!(
            key_action(KeyCode::Char('q'), KeyModifiers::empty()),
            KeyAction::Quit
        );
    }
}
