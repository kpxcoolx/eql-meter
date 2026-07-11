use super::{DamageEvent, LineData};
use once_cell::sync::Lazy;
use regex::Regex;

static MELEE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) (?P<hit>hit|hits|slash|slashes|crush|crushes|pierce|pierces|kick|kicks|bash|bashes|strike|strikes|claw|claws|bite|bites|punch|punches|backstab|backstabs|smite|smites|cleave|cleaves|frenzy|frenzies) (?P<target>.+?) for (?P<amount>\d+) points? of damage\.(?:\s*\((?P<mods>.+)\))?$",
    )
    .expect("melee regex")
});

// EQL: "You frenzy on a dar ghoul knight for 43 points of damage."
static FRENZY_ON_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) frenzy on (?P<target>.+?) for (?P<amount>\d+) points? of damage\.(?:\s*\((?P<mods>.+)\))?$",
    )
    .expect("frenzy on regex")
});

// Classic non-melee
static SPELL_DIRECT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) hit (?P<target>.+?) for (?P<amount>\d+) points? of non-melee damage\.(?:\s*\((?P<mods>.+)\))?$",
    )
    .expect("spell direct regex")
});

// EQL: "You hit a dar ghoul knight for 123 points of magic damage by Smiting Strike."
static SPELL_BY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?) hit (?P<target>.+?) for (?P<amount>\d+) points? of (?P<resist>\w+) damage by (?P<spell>.+?)\.(?:\s*\((?P<mods>.+)\))?$",
    )
    .expect("spell by regex")
});

// EMU / classic DoT style
static DOT_BY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<target>.+?) has taken (?P<amount>\d+) damage from (?P<spell>.+?) by (?P<attacker>.+?)\.$",
    )
    .expect("dot by regex")
});

// EQL: "A wan ghoul knight has taken 44 damage from your Blood Siphon Strike."
static YOUR_DOT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<target>.+?) has taken (?P<amount>\d+) damage from your (?P<spell>.+?)\.$",
    )
    .expect("your dot regex")
});

// "Francis's flame lick hits a goblin for 40 points of non-melee damage."
static POSSESSIVE_SPELL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?P<attacker>.+?)(?:'s|’) (?P<spell>.+?) (?P<hit>hit|hits) (?P<target>.+?) for (?P<amount>\d+) points? of (?:non-melee )?damage\.(?:\s*\((?P<mods>.+)\))?$",
    )
    .expect("possessive spell regex")
});

pub fn parse_damage_line(data: &LineData) -> Option<DamageEvent> {
    let action = data.action.as_str();

    if let Some(caps) = FRENZY_ON_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: "frenzy".to_string(),
            spell: None,
            modifiers: split_mods(caps.name("mods").map(|m| m.as_str())),
        });
    }

    if let Some(caps) = SPELL_BY_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: caps["resist"].to_ascii_lowercase(),
            spell: Some(caps["spell"].to_string()),
            modifiers: split_mods(caps.name("mods").map(|m| m.as_str())),
        });
    }

    if let Some(caps) = MELEE_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: normalize_hit_type(&caps["hit"]),
            spell: None,
            modifiers: split_mods(caps.name("mods").map(|m| m.as_str())),
        });
    }

    if let Some(caps) = SPELL_DIRECT_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: "non-melee".to_string(),
            spell: None,
            modifiers: split_mods(caps.name("mods").map(|m| m.as_str())),
        });
    }

    if let Some(caps) = POSSESSIVE_SPELL_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: "spell".to_string(),
            spell: Some(caps["spell"].to_string()),
            modifiers: split_mods(caps.name("mods").map(|m| m.as_str())),
        });
    }

    if let Some(caps) = DOT_BY_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: caps["attacker"].to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: "dot".to_string(),
            spell: Some(caps["spell"].to_string()),
            modifiers: Vec::new(),
        });
    }

    if let Some(caps) = YOUR_DOT_RE.captures(action) {
        return outgoing_or_none(DamageEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            incoming: false,
            attacker: "You".to_string(),
            target: caps["target"].to_string(),
            amount: caps["amount"].parse().ok()?,
            hit_type: "dot".to_string(),
            spell: Some(caps["spell"].to_string()),
            modifiers: Vec::new(),
        });
    }

    None
}

/// Incoming hits land on YOU — mark as taken, not DPS dealt.
fn outgoing_or_none(mut event: DamageEvent) -> Option<DamageEvent> {
    if event.target.eq_ignore_ascii_case("you") {
        event.incoming = true;
    }
    Some(event)
}

fn normalize_hit_type(raw: &str) -> String {
    let lower = raw.to_ascii_lowercase();
    match lower.as_str() {
        "hits" => "hit".to_string(),
        "slashes" => "slash".to_string(),
        "crushes" => "crush".to_string(),
        "pierces" => "pierce".to_string(),
        "kicks" => "kick".to_string(),
        "bashes" => "bash".to_string(),
        "strikes" => "strike".to_string(),
        "claws" => "claw".to_string(),
        "bites" => "bite".to_string(),
        "punches" => "punch".to_string(),
        "backstabs" => "backstab".to_string(),
        "smites" => "smite".to_string(),
        "cleaves" => "cleave".to_string(),
        "frenzies" => "frenzy".to_string(),
        other => other.to_string(),
    }
}

fn split_mods(raw: Option<&str>) -> Vec<String> {
    match raw {
        Some(s) => s
            .split_whitespace()
            .map(|part| part.trim_matches(|c: char| c == ',' || c == ';').to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        None => Vec::new(),
    }
}
