use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static WHO_LINE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?:AFK\s+)?\[(?P<level>\d+)\s+(?P<title>.+?)\s+\((?P<class>[^)]+)\)\]\s+(?P<name>\S+)(?:.*?\(Group:\s*(?P<group>\d+|None)\))?",
    )
    .expect("who regex")
});

static RAID_COUNT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^There are (?P<count>\d+) players? in your raid\.$").expect("raid count")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoPlayerEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub name: String,
    pub class_name: String,
    pub level: u32,
    /// 0 = None / ungrouped, 1+ = raid group number.
    pub group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidCountEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub count: u32,
}

pub fn parse_who_line(action: &str, timestamp: &str, time_secs: Option<f64>) -> Option<WhoPlayerEvent> {
    let caps = WHO_LINE.captures(action)?;
    let group = match caps.name("group").map(|m| m.as_str()) {
        Some("None") | None => 0,
        Some(raw) => raw.parse().unwrap_or(0),
    };
    Some(WhoPlayerEvent {
        timestamp: timestamp.to_string(),
        time_secs,
        name: caps["name"].to_string(),
        class_name: caps["class"].to_string(),
        level: caps["level"].parse().unwrap_or(0),
        group,
    })
}

pub fn parse_raid_count_line(
    action: &str,
    timestamp: &str,
    time_secs: Option<f64>,
) -> Option<RaidCountEvent> {
    let caps = RAID_COUNT.captures(action)?;
    Some(RaidCountEvent {
        timestamp: timestamp.to_string(),
        time_secs,
        count: caps["count"].parse().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grouped_who() {
        let e = parse_who_line(
            "[130 Juggernaut (Berserker)] Grudg (Group: 3)",
            "t",
            Some(1.0),
        )
        .expect("who");
        assert_eq!(e.name, "Grudg");
        assert_eq!(e.class_name, "Berserker");
        assert_eq!(e.group, 3);
        assert_eq!(e.level, 130);
    }

    #[test]
    fn parses_ungrouped_who() {
        let e = parse_who_line(
            "[130 Bloodreaver (Shadow Knight)] Waaine (Group: None)",
            "t",
            None,
        )
        .expect("who");
        assert_eq!(e.name, "Waaine");
        assert_eq!(e.group, 0);
    }

    #[test]
    fn parses_raid_count() {
        let e = parse_raid_count_line("There are 18 players in your raid.", "t", None)
            .expect("count");
        assert_eq!(e.count, 18);
    }
}
