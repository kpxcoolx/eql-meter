mod avoid;
mod damage;
mod heal;
mod misc;
mod stance;
mod who;

use chrono::{Local, NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

pub use avoid::{parse_avoid_line, parse_resist_line};
pub use damage::parse_damage_line;
pub use heal::parse_heal_line;
pub use misc::{parse_misc_line, MiscEvent};
pub use stance::{detect_stance_from_text, parse_stance_line, StanceEvent};
pub use who::{parse_raid_count_line, parse_who_line, RaidCountEvent, WhoPlayerEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum CombatEvent {
    Damage(DamageEvent),
    Death(DeathEvent),
    Avoid(AvoidEvent),
    Resist(ResistEvent),
    Heal(HealEvent),
    Stance(StanceEvent),
    Who(WhoPlayerEvent),
    RaidCount(RaidCountEvent),
    Misc(MiscEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub incoming: bool,
    pub attacker: String,
    pub target: String,
    pub amount: u64,
    pub hit_type: String,
    pub spell: Option<String>,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub target: String,
    pub killer: Option<String>,
    /// True when the logged character died (player death).
    pub self_death: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvoidEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub incoming: bool,
    pub attacker: String,
    pub target: String,
    pub attempt: String,
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResistEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub caster: String,
    pub spell: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub healer: String,
    pub target: String,
    pub amount: u64,
    pub overheal: u64,
    pub spell: String,
}

#[derive(Debug, Clone)]
pub struct LineData {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub action: String,
}

/// Strip EQ timestamp: `[Fri Jul 10 21:30:15 2026] message`
pub fn split_log_line(line: &str) -> Option<LineData> {
    let line = line.trim();
    if !line.starts_with('[') {
        return None;
    }

    let close = line.find(']')?;
    let timestamp = line[1..close].trim().to_string();
    let action = line[close + 1..].trim().to_string();
    if action.is_empty() {
        return None;
    }

    let time_secs = parse_eq_timestamp(&timestamp);
    Some(LineData {
        timestamp,
        time_secs,
        action,
    })
}

/// Parse `Fri Jul 10 21:30:15 2026` into unix seconds.
pub fn parse_eq_timestamp(ts: &str) -> Option<f64> {
    let naive = NaiveDateTime::parse_from_str(ts, "%a %b %d %H:%M:%S %Y").ok()?;
    let local = Local.from_local_datetime(&naive).single()?;
    Some(local.timestamp() as f64)
}

pub fn parse_line(line: &str) -> Option<CombatEvent> {
    let data = split_log_line(line)?;
    parse_action(&data)
}

pub fn parse_action(data: &LineData) -> Option<CombatEvent> {
    if let Some(stance) = parse_stance_line(data) {
        return Some(CombatEvent::Stance(stance));
    }
    if let Some(count) = parse_raid_count_line(&data.action, &data.timestamp, data.time_secs) {
        return Some(CombatEvent::RaidCount(count));
    }
    if let Some(who) = parse_who_line(&data.action, &data.timestamp, data.time_secs) {
        return Some(CombatEvent::Who(who));
    }
    if let Some(death) = parse_death(data) {
        return Some(CombatEvent::Death(death));
    }
    if let Some(heal) = parse_heal_line(data) {
        return Some(CombatEvent::Heal(heal));
    }
    if let Some(resist) = parse_resist_line(data) {
        return Some(CombatEvent::Resist(resist));
    }
    if let Some(avoid) = parse_avoid_line(data) {
        return Some(CombatEvent::Avoid(avoid));
    }
    if let Some(damage) = parse_damage_line(data) {
        return Some(CombatEvent::Damage(damage));
    }
    if let Some(self_hurt) = parse_self_hurt(data) {
        return Some(CombatEvent::Damage(self_hurt));
    }
    if let Some(misc) = parse_misc_line(&data.action, &data.timestamp, data.time_secs) {
        return Some(CombatEvent::Misc(misc));
    }
    None
}

fn parse_self_hurt(data: &LineData) -> Option<DamageEvent> {
    let lower = data.action.to_ascii_lowercase();
    let rest = lower.strip_prefix("you hurt yourself for ")?;
    let amount_str = rest.split_whitespace().next()?;
    let amount: u64 = amount_str.parse().ok()?;
    Some(DamageEvent {
        timestamp: data.timestamp.clone(),
        time_secs: data.time_secs,
        incoming: true,
        attacker: "Yourself".to_string(),
        target: "YOU".to_string(),
        amount,
        hit_type: "self".to_string(),
        spell: None,
        modifiers: Vec::new(),
    })
}

fn parse_death(data: &LineData) -> Option<DeathEvent> {
    let action = &data.action;
    let lower = action.to_ascii_lowercase();

    // Player death: "You have been slain by X!"
    if let Some(rest) = lower.strip_prefix("you have been slain by ") {
        let mut killer = action[action.len() - rest.len()..].trim().to_string();
        if killer.ends_with('!') {
            killer.pop();
        }
        return Some(DeathEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            target: "You".to_string(),
            killer: Some(killer),
            self_death: true,
        });
    }

    if lower == "you died." {
        return Some(DeathEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            target: "You".to_string(),
            killer: None,
            self_death: true,
        });
    }

    if let Some(rest) = action.strip_suffix(" died.") {
        return Some(DeathEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            target: rest.trim().to_string(),
            killer: None,
            self_death: false,
        });
    }

    // EQL: "You have slain a dar ghoul knight!"
    if let Some(rest) = lower.strip_prefix("you have slain ") {
        let mut target = action[action.len() - rest.len()..].trim().to_string();
        if target.ends_with('!') {
            target.pop();
        }
        return Some(DeathEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            target,
            killer: Some("You".to_string()),
            self_death: false,
        });
    }

    // Classic: "a goblin has been slain by Francis!"
    if let Some(idx) = lower.find(" has been slain by ") {
        let target = action[..idx].trim().to_string();
        let mut killer = action[idx + " has been slain by ".len()..].trim().to_string();
        if killer.ends_with('!') {
            killer.pop();
        }
        return Some(DeathEvent {
            timestamp: data.timestamp.clone(),
            time_secs: data.time_secs,
            target,
            killer: Some(killer),
            self_death: false,
        });
    }

    None
}

/// Character name from `eqlog_Name_Server.txt`
pub fn character_from_path(path: &str) -> Option<String> {
    let (name, _) = character_and_server_from_path(path)?;
    Some(name)
}

/// Server name from `eqlog_Name_Server.txt`
pub fn server_from_path(path: &str) -> Option<String> {
    let (_, server) = character_and_server_from_path(path)?;
    Some(server)
}

fn character_and_server_from_path(path: &str) -> Option<(String, String)> {
    let file = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let stem = file.strip_suffix(".txt").unwrap_or(file);
    let rest = stem.strip_prefix("eqlog_")?;
    let mut parts = rest.splitn(2, '_');
    let name = parts.next()?.to_string();
    let server = parts.next()?.to_string();
    if name.is_empty() || server.is_empty() {
        return None;
    }
    Some((name, server))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_eql_stance() {
        let stance =
            parse_line("[Fri Jul 10 21:24:59 2026] You assume a berserker stance.").expect("parse");
        match stance {
            CombatEvent::Stance(s) => {
                assert_eq!(s.name, "Berserker");
            }
            _ => panic!("expected stance"),
        }

        // Invocations are ignored — not a reliable class signal.
        assert!(parse_line(
            "[Fri Jul 10 21:24:59 2026] You begin reciting the spellblade invocation.",
        )
        .is_none());
    }

    #[test]
    fn detects_stance_from_kenkyo_log() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../samples/eqlog_Kenkyo_freeport.txt"
        );
        let text = std::fs::read_to_string(path).expect("read kenkyo log");
        let stance = detect_stance_from_text(&text);
        assert_eq!(stance.as_deref(), Some("Berserker"));
    }

    #[test]
    fn parses_melee_hit() {
        let line = "[Fri Jul 10 21:30:15 2026] You hit a goblin for 45 points of damage.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert_eq!(d.attacker, "You");
                assert_eq!(d.target, "a goblin");
                assert_eq!(d.amount, 45);
                assert_eq!(d.hit_type, "hit");
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_outgoing_miss() {
        let line =
            "[Fri Jul 10 21:25:07 2026] You try to slash a wan ghoul knight, but miss!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Avoid(a) => {
                assert!(!a.incoming);
                assert_eq!(a.outcome, "miss");
                assert_eq!(a.attempt, "slash");
                assert_eq!(a.target, "a wan ghoul knight");
            }
            _ => panic!("expected avoid"),
        }
    }

    #[test]
    fn parses_outgoing_dodge() {
        let line = "[Fri Jul 10 21:25:10 2026] You try to backstab a wan ghoul knight, but a wan ghoul knight dodges!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Avoid(a) => {
                assert_eq!(a.outcome, "dodge");
                assert_eq!(a.attempt, "backstab");
            }
            _ => panic!("expected avoid"),
        }
    }

    #[test]
    fn parses_incoming_defense() {
        let line =
            "[Fri Jul 10 21:25:21 2026] A dar ghoul knight tries to slash YOU, but YOU dodge!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Avoid(a) => {
                assert!(a.incoming);
                assert_eq!(a.outcome, "dodge");
            }
            _ => panic!("expected avoid"),
        }
    }

    #[test]
    fn parses_resist() {
        let line =
            "[Fri Jul 10 21:25:14 2026] You resist a wan ghoul knight's Ghoul Root!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Resist(r) => {
                assert_eq!(r.spell, "Ghoul Root");
                assert_eq!(r.caster, "a wan ghoul knight");
            }
            _ => panic!("expected resist"),
        }
    }

    #[test]
    fn parses_heal_and_overheal() {
        let line =
            "[Fri Jul 10 21:26:51 2026] You healed Kenkyo for 141 (399) hit points by Greater Healing.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Heal(h) => {
                assert_eq!(h.amount, 141);
                assert_eq!(h.overheal, 258);
                assert_eq!(h.spell, "Greater Healing");
            }
            _ => panic!("expected heal"),
        }
    }

    #[test]
    fn parses_self_hurt() {
        let line = "[Fri Jul 10 21:25:09 2026] You hurt yourself for 3 points.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert!(d.incoming);
                assert_eq!(d.amount, 3);
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_player_hit_with_crit() {
        let line =
            "[Fri Jul 10 21:30:16 2026] Francis crushes a goblin for 220 points of damage. (Critical)";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert_eq!(d.attacker, "Francis");
                assert_eq!(d.amount, 220);
                assert!(d.modifiers.iter().any(|m| m == "Critical"));
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_dot() {
        let line = "[Fri Jul 10 21:30:17 2026] A goblin has taken 55 damage from Flame Lick by Francis.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert_eq!(d.attacker, "Francis");
                assert_eq!(d.spell.as_deref(), Some("Flame Lick"));
                assert_eq!(d.amount, 55);
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_eql_spell_by() {
        let line = "[Fri Jul 10 21:25:12 2026] You hit a wan ghoul knight for 123 points of magic damage by Smiting Strike.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert_eq!(d.amount, 123);
                assert_eq!(d.hit_type, "magic");
                assert_eq!(d.spell.as_deref(), Some("Smiting Strike"));
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_eql_frenzy_on() {
        let line =
            "[Fri Jul 10 21:25:12 2026] You frenzy on a wan ghoul knight for 43 points of damage.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert_eq!(d.hit_type, "frenzy");
                assert_eq!(d.amount, 43);
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_incoming_damage() {
        let line =
            "[Fri Jul 10 21:25:06 2026] A wan ghoul knight hits YOU for 10 points of damage.";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Damage(d) => {
                assert!(d.incoming);
                assert_eq!(d.amount, 10);
                assert_eq!(d.attacker, "A wan ghoul knight");
            }
            _ => panic!("expected damage"),
        }
    }

    #[test]
    fn parses_eql_you_have_slain() {
        let line = "[Fri Jul 10 21:25:21 2026] You have slain a wan ghoul knight!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Death(d) => {
                assert_eq!(d.target, "a wan ghoul knight");
                assert_eq!(d.killer.as_deref(), Some("You"));
                assert!(!d.self_death);
            }
            _ => panic!("expected death"),
        }
    }

    #[test]
    fn parses_slain() {
        let line = "[Fri Jul 10 21:30:20 2026] A goblin has been slain by Francis!";
        let event = parse_line(line).expect("parse");
        match event {
            CombatEvent::Death(d) => {
                assert_eq!(d.target, "A goblin");
                assert_eq!(d.killer.as_deref(), Some("Francis"));
            }
            _ => panic!("expected death"),
        }
    }

    #[test]
    fn character_from_eqlog_name() {
        assert_eq!(
            character_from_path(r"C:\EQ\Logs\eqlog_Francis_legends.txt").as_deref(),
            Some("Francis")
        );
        assert_eq!(
            character_from_path(r"C:\Logs\eqlog_Kenkyo_freeport.txt").as_deref(),
            Some("Kenkyo")
        );
        assert_eq!(
            server_from_path(r"C:\Logs\eqlog_Kenkyo_freeport.txt").as_deref(),
            Some("freeport")
        );
    }

    #[test]
    fn parses_kenkyo_sample_log() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../samples/eqlog_Kenkyo_freeport.txt"
        );
        let text = std::fs::read_to_string(path).expect("read kenkyo log");
        let mut damage_events = 0u64;
        let mut death_events = 0u64;
        let mut avoid_events = 0u64;
        let mut heal_events = 0u64;
        let mut resist_events = 0u64;
        let mut total_damage = 0u64;

        for line in text.lines() {
            match parse_line(line) {
                Some(CombatEvent::Damage(d)) => {
                    damage_events += 1;
                    total_damage += d.amount;
                }
                Some(CombatEvent::Death(_)) => death_events += 1,
                Some(CombatEvent::Avoid(_)) => avoid_events += 1,
                Some(CombatEvent::Heal(_)) => heal_events += 1,
                Some(CombatEvent::Resist(_)) => resist_events += 1,
                Some(CombatEvent::Stance(_))
                | Some(CombatEvent::Who(_))
                | Some(CombatEvent::RaidCount(_))
                | Some(CombatEvent::Misc(_))
                | None => {}
            }
        }

        assert!(damage_events > 500, "expected many damage events, got {damage_events}");
        assert!(death_events >= 20, "expected many deaths, got {death_events}");
        assert!(avoid_events > 100, "expected many avoids, got {avoid_events}");
        assert!(heal_events > 10, "expected heals, got {heal_events}");
        assert!(resist_events > 0, "expected resists, got {resist_events}");
        assert!(total_damage > 50_000, "expected substantial damage, got {total_damage}");
        eprintln!(
            "Kenkyo log: {damage_events} hits, {avoid_events} avoids, {heal_events} heals, {resist_events} resists, {death_events} kills, {total_damage} total damage"
        );
    }
}
