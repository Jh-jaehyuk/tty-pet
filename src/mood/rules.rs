use crate::mood::Mood;

#[derive(Debug, Clone, Default)]
pub struct ObservedState {
    pub recent_event_kind: Option<String>,
    pub recent_event_age_secs: Option<u64>,
    pub dirty_count: Option<usize>,
    pub focus_minutes: u64,
}

pub fn evaluate(state: &ObservedState) -> Mood {
    if recent_event(state, "test_fail", 120) {
        return Mood::Worried;
    }

    if recent_event(state, "test_pass", 90) {
        return Mood::Happy;
    }

    if state.dirty_count.unwrap_or(0) >= 10 {
        return Mood::Busy;
    }

    if state.dirty_count.unwrap_or(0) >= 3 {
        return Mood::Playful;
    }

    if state.focus_minutes >= 90 {
        return Mood::Sleepy;
    }

    if state.dirty_count == Some(0) {
        return Mood::Calm;
    }

    Mood::Idle
}

fn recent_event(state: &ObservedState, kind: &str, max_age_secs: u64) -> bool {
    matches!(
        (&state.recent_event_kind, state.recent_event_age_secs),
        (Some(event_kind), Some(age)) if event_kind == kind && age <= max_age_secs
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_fail_beats_dirty_count() {
        let observed = ObservedState {
            recent_event_kind: Some("test_fail".to_string()),
            recent_event_age_secs: Some(30),
            dirty_count: Some(12),
            focus_minutes: 0,
        };

        assert_eq!(evaluate(&observed), Mood::Worried);
    }

    #[test]
    fn recent_pass_beats_dirty_count() {
        let observed = ObservedState {
            recent_event_kind: Some("test_pass".to_string()),
            recent_event_age_secs: Some(30),
            dirty_count: Some(12),
            focus_minutes: 0,
        };

        assert_eq!(evaluate(&observed), Mood::Happy);
    }

    #[test]
    fn dirty_count_beats_sleepy() {
        let observed = ObservedState {
            recent_event_kind: None,
            recent_event_age_secs: None,
            dirty_count: Some(10),
            focus_minutes: 120,
        };

        assert_eq!(evaluate(&observed), Mood::Busy);
    }

    #[test]
    fn clean_repo_is_calm() {
        let observed = ObservedState {
            dirty_count: Some(0),
            ..ObservedState::default()
        };

        assert_eq!(evaluate(&observed), Mood::Calm);
    }
}
