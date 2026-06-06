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
use crate::mood::phrases;
use crate::mood::rules::{self, ObservedState};
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
                if key.kind == KeyEventKind::Press
                    && (key.code == KeyCode::Char('q')
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)))
                {
                    break;
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
            let observed = ObservedState {
                recent_event_kind: latest_event.map(|event| event.kind),
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
            state.phrase = phrases::phrase_for(mood, state.frame).to_string();
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
