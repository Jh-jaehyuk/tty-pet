use crate::mood::Mood;

pub fn phrase_for(mood: Mood, frame: usize) -> &'static str {
    let phrases = match mood {
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
    };

    phrases[frame % phrases.len()]
}
