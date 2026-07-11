use super::{AvoidEvent, LineData, ResistEvent};
use once_cell::sync::Lazy;
use regex::Regex;

// You try to slash a wan ghoul knight, but miss!
// You try to frenzy on a wan ghoul knight, but miss!
static OUT_MISS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^You try to (?:frenzy on |(?P<attempt>[\w]+) )?(?P<target>.+?), but miss!$",
    )
    .expect("out miss regex")
});

// You try to backstab a wan ghoul knight, but a wan ghoul knight dodges!
static OUT_AVOID_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^You try to (?:frenzy on |(?P<attempt>[\w]+) )?(?P<target>.+?), but .+? (?P<outcome>dodges|parries|blocks)!$",
    )
    .expect("out avoid regex")
});

// A wan ghoul knight tries to cleave YOU, but misses!
static IN_MISS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) tries to (?P<attempt>[\w]+) YOU, but misses!(?:\s*\(.*\))?$",
    )
    .expect("in miss regex")
});

// A dar ghoul knight tries to slash YOU, but YOU dodge!
static IN_DEFENSE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) tries to (?P<attempt>[\w]+) YOU, but YOU (?P<outcome>dodge|parry|block|riposte)!$",
    )
    .expect("in defense regex")
});

// You resist a wan ghoul knight's Ghoul Root!
static RESIST_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^You resist (?P<caster>.+?)(?:'s|’) (?P<spell>.+?)!$").expect("resist regex")
});

fn normalize_outcome(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "dodges" | "dodge" => "dodge".to_string(),
        "parries" | "parry" => "parry".to_string(),
        "blocks" | "block" => "block".to_string(),
        "riposte" => "riposte".to_string(),
        "miss" | "misses" => "miss".to_string(),
        other => other.to_string(),
    }
}

pub fn parse_avoid_line(data: &LineData) -> Option<AvoidEvent> {
    let action = data.action.as_str();

    if let Some(caps) = OUT_MISS_RE.captures(action) {
        let attempt = caps
            .name("attempt")
            .map(|m| m.as_str().to_ascii_lowercase())
            .unwrap_or_else(|| "frenzy".to_string());
        return Some(AvoidEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: "You".to_string(),
            target: caps["target"].to_string(),
            attempt,
            outcome: "miss".to_string(),
        });
    }

    if let Some(caps) = OUT_AVOID_RE.captures(action) {
        let attempt = caps
            .name("attempt")
            .map(|m| m.as_str().to_ascii_lowercase())
            .unwrap_or_else(|| "frenzy".to_string());
        return Some(AvoidEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: "You".to_string(),
            target: caps["target"].to_string(),
            attempt,
            outcome: normalize_outcome(&caps["outcome"]),
        });
    }

    if let Some(caps) = IN_MISS_RE.captures(action) {
        return Some(AvoidEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: true,
            attacker: caps["attacker"].to_string(),
            target: "YOU".to_string(),
            attempt: caps["attempt"].to_ascii_lowercase(),
            outcome: "miss".to_string(),
        });
    }

    if let Some(caps) = IN_DEFENSE_RE.captures(action) {
        return Some(AvoidEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: true,
            attacker: caps["attacker"].to_string(),
            target: "YOU".to_string(),
            attempt: caps["attempt"].to_ascii_lowercase(),
            outcome: normalize_outcome(&caps["outcome"]),
        });
    }

    None
}

pub fn parse_resist_line(data: &LineData) -> Option<ResistEvent> {
    let caps = RESIST_RE.captures(&data.action)?;
    Some(ResistEvent {
        timestamp: data.timestamp.clone(),
        time_secs: data.time_secs,
        caster: caps["caster"].to_string(),
        spell: caps["spell"].to_string(),
    })
}
