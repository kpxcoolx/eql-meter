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

/// Private pet tell: `Garn told you, 'Attacking Lord Nagafen Master.'`
/// also `… 'I am unable to wake a goblin, Master.'`
static PET_CLAIM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)^(?P<pet>\S+) told you, ['\"](?:Attacking (?P<target>.+) Master|I am unable to wake (?P<wake>.+?), Master)\.['\"]$"#,
    )
    .expect("pet claim")
});

/// `/pet who leader` answer: `Jaber says, 'My leader is Kenkyo.'`
static PET_LEADER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?i)^(?P<pet>.+?) says, ['\"]My leader is (?P<leader>.+?)\.['\"]$"#,
    )
    .expect("pet leader")
});

/// First-person cast: `You begin casting Burnout.`
static CAST_BEGIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^You begin casting (?P<spell>.+?)\.$").expect("cast begin")
});

/// Burnout landing: `Jabektik goes berserk.`
static PET_BERSERK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<who>.+?) goes berserk\.$").expect("pet berserk")
});

static GROUP_JOIN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<who>.+?) has joined the group\.$").expect("group join")
});

static GROUP_LEAVE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<who>.+?) has left the group\.$").expect("group leave")
});

static GROUP_REMOVE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^You (?:remove|have removed) (?P<who>.+?) from the group\.?$")
        .expect("group remove")
});

static GROUP_LEADER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(?P<who>.+?) is now the leader of your group\.$").expect("group leader")
});

static YOU_JOIN_GROUP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^You have joined the group\.$").expect("you join group")
});

static GROUP_DISBAND: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(?:Your group has been disbanded|You disband your group|You have (?:left|been removed from) the group)\.$",
    )
    .expect("group disband")
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MiscKind {
    Loot,
    Random,
    Roll,
    Chat,
    /// Named pet identified you as owner (attack tell or failed-wake tell).
    PetEngage,
    /// `/pet who leader` public answer.
    PetLeader,
    /// `You begin casting Burnout.` (and other pet-only spells).
    CastBegin,
    /// `Jabektik goes berserk.` — Burnout landing on a named pet.
    PetBuffLanding,
    GroupJoin,
    GroupLeave,
    GroupDisband,
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

pub fn is_pet_buff_spell(spell: &str) -> bool {
    let lower = spell.trim().to_ascii_lowercase();
    lower.contains("burnout")
        || lower.contains("augment death")
        || lower.contains("focus death")
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
    if let Some(caps) = PET_CLAIM.captures(action) {
        let target = caps
            .name("target")
            .or_else(|| caps.name("wake"))
            .map(|m| m.as_str().to_string());
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::PetEngage,
            summary: format!("{} engaging {}", &caps["pet"], target.as_deref().unwrap_or("")),
            who: Some(caps["pet"].to_string()),
            detail: target,
        });
    }
    if let Some(caps) = PET_LEADER.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::PetLeader,
            summary: format!("{} leader {}", &caps["pet"], &caps["leader"]),
            who: Some(caps["pet"].to_string()),
            detail: Some(caps["leader"].to_string()),
        });
    }
    if let Some(caps) = CAST_BEGIN.captures(action) {
        let spell = caps["spell"].to_string();
        if is_pet_buff_spell(&spell) {
            return Some(MiscEvent {
                timestamp: timestamp.to_string(),
                time_secs,
                kind: MiscKind::CastBegin,
                summary: format!("Casting {spell}"),
                who: None,
                detail: Some(spell),
            });
        }
    }
    if let Some(caps) = PET_BERSERK.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::PetBuffLanding,
            summary: format!("{} goes berserk", &caps["who"]),
            who: Some(caps["who"].to_string()),
            detail: Some("berserk".to_string()),
        });
    }
    if YOU_JOIN_GROUP.is_match(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupJoin,
            summary: action.to_string(),
            who: Some("You".to_string()),
            detail: None,
        });
    }
    if GROUP_DISBAND.is_match(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupDisband,
            summary: action.to_string(),
            who: None,
            detail: None,
        });
    }
    if let Some(caps) = GROUP_JOIN.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupJoin,
            summary: format!("{} joined group", &caps["who"]),
            who: Some(caps["who"].to_string()),
            detail: None,
        });
    }
    if let Some(caps) = GROUP_LEADER.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupJoin,
            summary: format!("{} group leader", &caps["who"]),
            who: Some(caps["who"].to_string()),
            detail: Some("leader".to_string()),
        });
    }
    if let Some(caps) = GROUP_LEAVE.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupLeave,
            summary: format!("{} left group", &caps["who"]),
            who: Some(caps["who"].to_string()),
            detail: None,
        });
    }
    if let Some(caps) = GROUP_REMOVE.captures(action) {
        return Some(MiscEvent {
            timestamp: timestamp.to_string(),
            time_secs,
            kind: MiscKind::GroupLeave,
            summary: format!("{} removed from group", &caps["who"]),
            who: Some(caps["who"].to_string()),
            detail: None,
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

    #[test]
    fn parses_pet_engage_tell() {
        let engage = parse_misc_line(
            "Garn told you, 'Attacking Lord Nagafen Master.'",
            "t",
            Some(1.0),
        )
        .unwrap();
        assert_eq!(engage.kind, MiscKind::PetEngage);
        assert_eq!(engage.who.as_deref(), Some("Garn"));
        assert_eq!(engage.detail.as_deref(), Some("Lord Nagafen"));
    }

    #[test]
    fn parses_pet_unable_wake_tell() {
        let claim = parse_misc_line(
            "Garn told you, 'I am unable to wake a skeleton, Master.'",
            "t",
            None,
        )
        .unwrap();
        assert_eq!(claim.kind, MiscKind::PetEngage);
        assert_eq!(claim.who.as_deref(), Some("Garn"));
    }

    #[test]
    fn parses_pet_leader_say() {
        let leader = parse_misc_line("Jaber says, 'My leader is Kenkyo.'", "t", None).unwrap();
        assert_eq!(leader.kind, MiscKind::PetLeader);
        assert_eq!(leader.who.as_deref(), Some("Jaber"));
        assert_eq!(leader.detail.as_deref(), Some("Kenkyo"));
    }

    #[test]
    fn parses_burnout_cast_and_landing() {
        let cast = parse_misc_line("You begin casting Burnout IV.", "t", None).unwrap();
        assert_eq!(cast.kind, MiscKind::CastBegin);
        assert_eq!(cast.detail.as_deref(), Some("Burnout IV"));

        let land = parse_misc_line("Jabektik goes berserk.", "t", None).unwrap();
        assert_eq!(land.kind, MiscKind::PetBuffLanding);
        assert_eq!(land.who.as_deref(), Some("Jabektik"));

        assert!(parse_misc_line("You begin casting Complete Healing.", "t", None).is_none());
    }

    #[test]
    fn parses_group_join_and_leave() {
        let join = parse_misc_line("Galactic has joined the group.", "t", None).unwrap();
        assert_eq!(join.kind, MiscKind::GroupJoin);
        assert_eq!(join.who.as_deref(), Some("Galactic"));

        let leave = parse_misc_line("Galactic has left the group.", "t", None).unwrap();
        assert_eq!(leave.kind, MiscKind::GroupLeave);

        let disband = parse_misc_line("Your group has been disbanded.", "t", None).unwrap();
        assert_eq!(disband.kind, MiscKind::GroupDisband);

        let you_join = parse_misc_line("You have joined the group.", "t", None).unwrap();
        assert_eq!(you_join.kind, MiscKind::GroupJoin);
    }
}
