pub mod phrases;
pub mod rules;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Idle,
    Calm,
    Playful,
    Happy,
    Worried,
    Busy,
    Sleepy,
}

impl Mood {
    pub fn as_str(self) -> &'static str {
        match self {
            Mood::Idle => "idle",
            Mood::Calm => "calm",
            Mood::Playful => "playful",
            Mood::Happy => "happy",
            Mood::Worried => "worried",
            Mood::Busy => "busy",
            Mood::Sleepy => "sleepy",
        }
    }
}
