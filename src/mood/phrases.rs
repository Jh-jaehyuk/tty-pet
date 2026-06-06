use crate::mood::Mood;

pub fn phrase_for(mood: Mood, frame: usize) -> &'static str {
    phrase_for_event(mood, None, frame)
}

pub fn phrase_for_event(mood: Mood, event_kind: Option<&str>, frame: usize) -> &'static str {
    let phrases = match event_kind {
        Some("poke") => &["boop?", "tiny jump.", "hey, paws off."][..],
        Some("treat") => &["cronch.", "snack acquired.", "repo snack accepted."][..],
        Some("call") => &["coming.", "hop hop.", "you rang?"][..],
        Some("nap") => &["tiny nap.", "soft compile dreams.", "zZ"][..],
        _ => phrases_for_mood(mood),
    };

    phrases[frame % phrases.len()]
}

fn phrases_for_mood(mood: Mood) -> &'static [&'static str] {
    match mood {
        Mood::Idle => &["boop.", "watching the repo.", "tiny paws online."][..],
        Mood::Calm => &[
            "clean little den.",
            "repo smells tidy.",
            "soft paws, clean tree.",
        ][..],
        Mood::Playful => &["tiny paws, big diff.", "hop hop.", "this repo wiggles."][..],
        Mood::Happy => &["green feels crunchy.", "ship snack?", "nice checkpoint."][..],
        Mood::Worried => &["uh oh.", "the red thing blinked.", "poke it again?"][..],
        Mood::Busy => &[
            "many files, many hops.",
            "diff zoomies.",
            "busy little paws.",
        ][..],
        Mood::Sleepy => &["tiny stretch.", "repo nap?", "blink blink."][..],
    }
}
