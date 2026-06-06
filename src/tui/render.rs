use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::pet::built_in;
use crate::tui::app_state::WatchState;

pub fn render(frame: &mut Frame<'_>, state: &WatchState) {
    let area = frame.size();
    let block = Block::default().borders(Borders::ALL).title(" tty-pet ");
    let inner = block.inner(area);

    frame.render_widget(block, area);

    if inner.width < 8 || inner.height < 3 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let dirty = state
        .dirty_count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "?".to_string());
    let test = state.last_test_status.as_deref().unwrap_or("?");
    let header = Line::from(vec![
        Span::styled(&state.project_name, Style::default().fg(Color::Cyan)),
        Span::raw(format!("  git:{dirty}  test:{test}  bond:{}", state.bond)),
    ]);
    frame.render_widget(Paragraph::new(header), chunks[0]);

    let sprite = sprite_body(state);
    let (sprite_width, sprite_height) = sprite_dimensions(&sprite);
    let pet_area = moving_pet_area(chunks[1], sprite_width, sprite_height, state.frame);
    let pet = Paragraph::new(sprite).style(Style::default().fg(Color::Yellow));
    frame.render_widget(pet, pet_area);

    let phrase = format!("{}: \"{}\"", state.mood.as_str(), state.phrase);
    let footer = Paragraph::new(phrase)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(footer, chunks[2]);
}

fn sprite_body(state: &WatchState) -> String {
    state.custom_sprite.as_ref().map_or_else(
        || built_in::sprite_for(state.mood, state.frame).join("\n"),
        |lines| lines.join("\n"),
    )
}

fn moving_pet_area(area: Rect, sprite_width: u16, sprite_height: u16, frame: usize) -> Rect {
    let width = sprite_width.min(area.width);
    let height = sprite_height.min(area.height);
    let x_offset = bouncing_offset(frame, area.width, width);
    let y_offset = bouncing_offset(frame / 2 + 3, area.height, height);

    Rect {
        x: area.x + x_offset,
        y: area.y + y_offset,
        width,
        height,
    }
}

fn bouncing_offset(frame: usize, area_width: u16, sprite_width: u16) -> u16 {
    let available = area_width.saturating_sub(sprite_width);

    if available == 0 {
        return 0;
    }

    let period = usize::from(available) * 2;
    let progress = frame % period;
    let offset = if progress <= usize::from(available) {
        progress
    } else {
        period - progress
    };

    offset as u16
}

fn sprite_dimensions(body: &str) -> (u16, u16) {
    let width = body
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or_default()
        .min(usize::from(u16::MAX)) as u16;
    let height = body.lines().count().min(usize::from(u16::MAX)) as u16;

    (width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bouncing_offset_uses_available_width() {
        assert_eq!(bouncing_offset(0, 20, 5), 0);
        assert_eq!(bouncing_offset(7, 20, 5), 7);
        assert_eq!(bouncing_offset(15, 20, 5), 15);
        assert_eq!(bouncing_offset(20, 20, 5), 10);
        assert_eq!(bouncing_offset(30, 20, 5), 0);
    }

    #[test]
    fn bouncing_offset_stays_zero_when_sprite_does_not_fit() {
        assert_eq!(bouncing_offset(10, 5, 10), 0);
        assert_eq!(bouncing_offset(10, 5, 5), 0);
    }

    #[test]
    fn moving_pet_area_keeps_sprite_inside_viewport() {
        let area = Rect {
            x: 2,
            y: 3,
            width: 30,
            height: 8,
        };
        let pet_area = moving_pet_area(area, 10, 2, 20);

        assert!(pet_area.x >= area.x);
        assert!(pet_area.x + pet_area.width <= area.x + area.width);
        assert!(pet_area.y >= area.y);
        assert!(pet_area.y + pet_area.height <= area.y + area.height);
    }

    #[test]
    fn moving_pet_area_uses_vertical_space_when_available() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 30,
            height: 10,
        };
        let first = moving_pet_area(area, 8, 2, 0);
        let later = moving_pet_area(area, 8, 2, 8);

        assert_ne!(first.y, later.y);
    }

    #[test]
    fn sprite_dimensions_use_widest_line() {
        assert_eq!(sprite_dimensions("cat\nlong cat"), (8, 2));
    }
}
