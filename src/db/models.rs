use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PetState {
    pub project_id: String,
    pub bond: i64,
    pub mood: String,
    pub last_test_status: Option<String>,
    pub last_event_kind: Option<String>,
    pub last_event_at: Option<String>,
    pub focus_started_at: Option<String>,
    pub updated_at: String,
    pub custom_image: Option<CustomImageConfig>,
}

#[derive(Debug, Clone)]
pub struct ProjectEvent {
    pub id: i64,
    pub project_id: String,
    pub kind: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomImageConfig {
    pub path: PathBuf,
    pub width: u32,
    pub height_scale: f32,
    pub charset: String,
    pub invert: bool,
}
