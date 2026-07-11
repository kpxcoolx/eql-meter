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
        target: caps["target"].to_string(),
        amount,
        overheal,
        spell: caps["spell"].to_string(),
    })
}
