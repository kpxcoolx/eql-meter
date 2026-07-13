use super::{HealEvent, LineData};
use once_cell::sync::Lazy;
use regex::Regex;

// You healed Kenkyo for 44 hit points by Blood Siphon Strike.
// You healed Kenkyo for 141 (399) hit points by Greater Healing.
static HEAL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<healer>.+?) healed (?P<target>.+?) for (?P<amount>\d+)(?: \((?P<full>\d+)\))? hit points? by (?P<spell>.+?)\.$",
    )
    .expect("heal regex")
});

pub fn parse_heal_line(data: &LineData) -> Option<HealEvent> {
    let caps = HEAL_RE.captures(&data.action)?;
    let amount: u64 = caps["amount"].parse().ok()?;
    let full = caps
        .name("full")
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(amount);
    let overheal = full.saturating_sub(amount);

    Some(HealEvent {
        timestamp: data.timestamp.clone(),
        time_secs: data.time_secs,
        healer: caps["healer"].to_string(),
        // HoT lines look like "You healed Kenkyo over time for …" — don't keep
        // "Kenkyo over time" as a fake combatant name.
        target: strip_over_time_suffix(&caps["target"]),
        amount,
        overheal,
        spell: caps["spell"].to_string(),
    })
}

fn strip_over_time_suffix(name: &str) -> String {
    let trimmed = name.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(base) = lower.strip_suffix(" over time") {
        let end = base.len();
        return trimmed[..end].trim().to_string();
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::LineData;

    #[test]
    fn strips_over_time_from_heal_target() {
        let data = LineData {
            timestamp: String::new(),
            time_secs: Some(1.0),
            action: "You healed Kenkyo over time for 44 hit points by Blood Siphon Strike."
                .into(),
        };
        let heal = parse_heal_line(&data).expect("heal");
        assert_eq!(heal.target, "Kenkyo");
        assert_eq!(heal.healer, "You");
        assert_eq!(heal.amount, 44);
    }
}
