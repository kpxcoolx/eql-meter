use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static LOOT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<who>.+?) (?:looted|has looted) (?:a |an |)?(?P<item>.+?)(?: from .+)?\.$",
    )
    .expect("loot")
});

static RANDOM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\*\*Random(?: Number)?:\s*(?P<low>\d+)\s+to\s+(?P<high>\d+)\*\*$")
        .expect("random")
});

static RANDOM_ROLL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<who>\S+) rolls a?\s*(?P<roll>\d+)(?:\s*\((?P<low>\d+)-(?P<high>\d+)\))?$")
        .expect("roll")
});

static CHAT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)^(?P<who>\S+) (?:tells? (?:the )?(?P<channel>group|raid|guild|say|ooc|shout)|tells you|says?|shouts?),?\s*['"](?P<msg>.+)['"]$"#,
    )
    .expect("chat")
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MiscKind {
    Loot,
    Random,
    Roll,
    Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiscEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub kind: MiscKind,
    pub summary: String,
    pub who: Option<String>,
    pub detail: Option<String>,
}

pub fn parse_misc_line(action: &str, timestamp: &str, time_secs: Option<f64>) -> Option<MiscEvent> {
    if let Some(caps) = LOOT.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::Loot,
            summary: format!("{} looted {}", &caps["who"], &caps["item"]),
            who: Some(caps["who"].to_string()),
            detail: Some(caps["item"].to_string()),
        });
    }
    if let Some(caps) = RANDOM.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::Random,
            summary: format!("Random {}-{}", &caps["low"], &caps["high"]),
            who: None,
            detail: Some(format!("{}-{}", &caps["low"], &caps["high"])),
        });
    }
    if let Some(caps) = RANDOM_ROLL.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::Roll,
            summary: format!("{} rolled {}", &caps["who"], &caps["roll"]),
            who: Some(caps["who"].to_string()),
            detail: Some(caps["roll"].to_string()),
        });
    }
    if let Some(caps) = CHAT.captures(action) {
        let channel = caps
            .name("channel")
            .map(|m| m.as_str())
            .unwrap_or("chat");
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::Chat,
            summary: format!("{} ({}) {}", &caps["who"], channel, &caps["msg"]),
            who: Some(caps["who"].to_string()),
            detail: Some(caps["msg"].to_string()),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_loot_and_roll() {
        let loot = parse_misc_line(
            "Francis looted a Platinum Ring from a goblin corpse.",
            "Mon Jul 10 12:00:00 2026",
            Some(1.0),
        )
        .unwrap();
        assert_eq!(loot.kind, MiscKind::Loot);
        assert_eq!(loot.detail.as_deref(), Some("Platinum Ring"));

        let roll = parse_misc_line("Bob rolls 87 (0-100)", "t", None).unwrap();
        assert_eq!(roll.kind, MiscKind::Roll);
        assert_eq!(roll.detail.as_deref(), Some("87"));
    }

    #[test]
    fn parses_chat() {
        let chat = parse_misc_line("Alice tells the raid, 'stack on me'", "t", None).unwrap();
        assert_eq!(chat.kind, MiscKind::Chat);
        assert!(chat.summary.contains("raid"));
    }
}
