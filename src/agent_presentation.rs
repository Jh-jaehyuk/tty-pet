use serde_json::{json, Value};

const RECENT_EVENT_SECONDS: i64 = 120;

pub fn event_payload(event_kind: &str, status: &Value) -> Value {
    let reaction = reaction_for_event(event_kind);

    json!({
        "event": {
            "kind": event_kind,
            "recorded_kind": status["state"]["last_event"]["kind"].clone()
        },
        "reaction": reaction,
        "state": state_summary(status),
        "presentation": presentation_for_event(event_kind)
    })
}

pub fn status_payload(status: &Value) -> Value {
    let mood = status["state"]["mood"].as_str().unwrap_or("idle");

    json!({
        "reaction": {
            "mood": status["state"]["mood"].clone(),
            "phrase": phrase_for_mood(mood),
            "motion": "idle",
            "face": face_for_mood(mood)
        },
        "state": state_summary(status),
        "presentation": presentation_for_status(status)
    })
}

fn state_summary(status: &Value) -> Value {
    json!({
        "project": status["project"].clone(),
        "mood": status["state"]["mood"].clone(),
        "bond": status["state"]["bond"].clone(),
        "last_test_status": status["state"]["last_test_status"].clone(),
        "last_event_kind": status["state"]["last_event"]["kind"].clone(),
        "last_event_at": status["state"]["last_event"]["created_at"].clone(),
        "dirty_files": status["state"]["dirty_files"].clone(),
        "pet": status["pet"].clone()
    })
}

fn reaction_for_event(event_kind: &str) -> Value {
    let (mood, phrase, motion) = event_reaction_parts(event_kind);

    json!({
        "mood": mood,
        "phrase": phrase,
        "motion": motion,
        "face": face_for_mood(mood)
    })
}

fn event_reaction_parts(event_kind: &str) -> (&'static str, &'static str, &'static str) {
    match event_kind {
        "pass" => ("happy", "green feels crunchy.", "hop"),
        "fail" => ("worried", "the red thing blinked.", "small-flinch"),
        "poke" => ("playful", "boop?", "jump"),
        "treat" => ("happy", "snack acquired.", "hop"),
        "call" => ("playful", "you rang?", "come-over"),
        "nap" => ("sleepy", "tiny nap.", "curl-up"),
        _ => ("idle", "tiny paws online.", "idle"),
    }
}

fn presentation_for_event(event_kind: &str) -> Value {
    let (mood, _, _) = event_reaction_parts(event_kind);
    let (ko, en) = match event_kind {
        "pass" => (
            "pass 기록 완료. 펫이 초록 불빛을 보고 살짝 뛰었어요.",
            "Pass recorded. The pet did a tiny hop at the green light.",
        ),
        "fail" => (
            "fail 기록 완료. 펫이 잠깐 귀를 접었지만, 혼내지는 않아요.",
            "Fail recorded. The pet folded its ears for a moment, but it is not here to scold.",
        ),
        "poke" => (
            "poke 기록 완료. 펫이 깜짝 놀라서 작게 튀었어요.",
            "Poke recorded. The pet startled into a tiny jump.",
        ),
        "treat" => (
            "간식 전달 완료. 펫이 바로 튀어나와서 받아 갔어요.",
            "Treat delivered. The pet hopped over for the snack.",
        ),
        "call" => (
            "call 기록 완료. 펫이 부르는 소리를 듣고 이쪽으로 고개를 돌렸어요.",
            "Call recorded. The pet turned toward the sound.",
        ),
        "nap" => (
            "낮잠 기록 완료. 펫이 화면 한쪽에서 조용히 웅크렸어요.",
            "Nap recorded. The pet curled up quietly on one side of the screen.",
        ),
        _ => (
            "이벤트 기록 완료. 펫이 짧게 반응했어요.",
            "Event recorded. The pet gave a small reaction.",
        ),
    };

    presentation(ko, en, face_for_mood(mood))
}

fn presentation_for_status(status: &Value) -> Value {
    let mood = status["state"]["mood"].as_str().unwrap_or("idle");
    let bond = status["state"]["bond"].as_i64().unwrap_or(0);
    let (ko, en) = if let Some(event_kind) = recent_event_kind(status) {
        (
            format!("현재 펫은 {mood} 상태예요. 방금 {event_kind} 이벤트를 기억하고 짧게 반응하고 있습니다."),
            format!("The pet is currently {mood}. It still remembers the recent {event_kind} and is giving a small reaction."),
        )
    } else {
        (
            format!("현재 펫은 {mood} 상태예요. bond는 {bond}이고 조용히 프로젝트를 지켜보고 있습니다."),
            format!("The pet is currently {mood}. Bond is {bond}, and it is quietly watching the project."),
        )
    };

    presentation(&ko, &en, face_for_mood(mood))
}

fn recent_event_kind(status: &Value) -> Option<&str> {
    let event_kind = status["state"]["last_event"]["kind"].as_str()?;
    let event_at = status["state"]["last_event"]["created_at"]
        .as_str()?
        .parse::<i64>()
        .ok()?;
    let age = crate::time::now_unix_seconds().saturating_sub(event_at);

    (age <= RECENT_EVENT_SECONDS).then_some(event_kind)
}

fn presentation(ko: &str, en: &str, face: &[&str]) -> Value {
    let markdown = format!("```\n{}\n```\n{}", face.join("\n"), ko);

    json!({
        "ko": ko,
        "en": en,
        "markdown": markdown,
        "style": "short_playful",
        "rules": [
            "Do not invent state.",
            "Do not give development advice.",
            "Keep it to one or two short sentences.",
            "When the user wants to see the pet, include the face from presentation.markdown."
        ]
    })
}

fn phrase_for_mood(mood: &str) -> &'static str {
    match mood {
        "calm" => "repo smells tidy.",
        "playful" => "tiny paws, big diff.",
        "happy" => "green feels crunchy.",
        "worried" => "uh oh.",
        "busy" => "diff zoomies.",
        "sleepy" => "tiny stretch.",
        _ => "tiny paws online.",
    }
}

fn face_for_mood(mood: &str) -> &'static [&'static str] {
    match mood {
        "happy" => &["\\(=^o^=)/"],
        "worried" => &["(=;-;=)"],
        "sleepy" => &["(= -.-=) zZ"],
        "busy" => &["(=^._.^=) >", "  /   \\"],
        "playful" => &[" (=^._.^=)"],
        "calm" => &["(=^._.^=)"],
        _ => &["(=^._.^=)"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_presentation_ignores_old_events() {
        let status = json!({
            "project": {"name": "tty-pet"},
            "state": {
                "mood": "calm",
                "bond": 7,
                "last_test_status": "pass",
                "dirty_files": 0,
                "last_event": {
                    "kind": "treat",
                    "created_at": (crate::time::now_unix_seconds() - RECENT_EVENT_SECONDS - 1).to_string()
                }
            },
            "pet": {"image": {"kind": "built-in"}}
        });

        let payload = status_payload(&status);
        let ko = payload["presentation"]["ko"]
            .as_str()
            .expect("presentation should include Korean text");

        assert!(ko.contains("bond는 7"));
        assert!(!ko.contains("방금 treat"));
    }
}
