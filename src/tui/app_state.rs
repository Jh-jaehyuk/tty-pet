use std::time::Instant;

use crate::mood::Mood;

#[derive(Debug, Clone)]
pub struct WatchState {
    pub project_name: String,
    pub dirty_count: Option<usize>,
    pub mood: Mood,
    pub phrase: String,
    pub bond: i64,
    pub last_test_status: Option<String>,
    pub custom_sprite: Option<Vec<String>>,
    pub custom_sprite_key: Option<String>,
    pub frame: usize,
    pub started_at: Instant,
}

impl WatchState {
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            dirty_count: None,
            mood: Mood::Idle,
            phrase: "boop.".to_string(),
            bond: 0,
            last_test_status: None,
            custom_sprite: None,
            custom_sprite_key: None,
            frame: 0,
            started_at: Instant::now(),
        }
    }

    pub fn advance_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
    }

    pub fn focus_minutes(&self) -> u64 {
        self.started_at.elapsed().as_secs() / 60
    }
}
