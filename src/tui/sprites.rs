use crate::mood::Mood;

pub fn sprite_for(mood: Mood, frame: usize) -> &'static [&'static str] {
    match mood {
        Mood::Happy => &["\\(=^o^=)/"],
        Mood::Worried => &["(=;-;=)"],
        Mood::Sleepy => &["(= -.-=) zZ"],
        Mood::Busy => {
            if frame % 2 == 0 {
                &["(=^._.^=) >", "  /   \\"]
            } else {
                &["< (=^._.^=)", "  /   \\"]
            }
        }
        Mood::Playful => {
            if frame % 2 == 0 {
                &[" (=^._.^=)"]
            } else {
                &["  (=^._.^=)"]
            }
        }
        Mood::Calm => &["(=^._.^=)"],
        Mood::Idle => &["(=^._.^=)"],
    }
}
