use crate::parse::{
    AvoidEvent, CombatEvent, DamageEvent, DeathEvent, HealEvent, MiscEvent, ResistEvent,
    WhoPlayerEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const IDLE_SECS: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityStat {
    pub name: String,
    pub hits: u64,
    pub damage: u64,
    pub healing: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    pub sec: u32,
    pub damage: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStat {
    pub name: String,
    pub damage: u64,
    pub hits: u64,
    pub pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStat {
    pub name: String,
    pub damage: u64,
    pub hits: u64,
    pub crits: u64,
    pub max_hit: u64,
    pub dps: f64,
    pub pct: f64,
    pub attempts: u64,
    pub misses: u64,
    pub accuracy_pct: f64,
    pub healing: u64,
    pub overheal: u64,
    pub hps: f64,
    pub heal_pct: f64,
    pub healing_received: u64,
    pub abilities: Vec<AbilityStat>,
    pub timeline: Vec<TimelinePoint>,
    pub heal_timeline: Vec<TimelinePoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FightSummary {
    pub id: u64,
    pub target: String,
    /// Damage dealt to each NPC in this fight/combined view.
    pub targets: Vec<TypeStat>,
    pub started_at: f64,
    pub ended_at: Option<f64>,
    pub duration_secs: f64,
    pub total_damage: u64,
    pub total_dps: f64,
    pub peak_dps: f64,
    pub total_hits: u64,
    pub crits: u64,
    pub crit_pct: f64,
    pub max_hit: u64,
    pub max_hit_by: Option<String>,
    pub damage_taken: u64,
    pub taken_hits: u64,
    pub dtps: f64,
    pub max_taken_hit: u64,
    pub attempts: u64,
    pub misses: u64,
    pub accuracy_pct: f64,
    pub dodges: u64,
    pub parries: u64,
    pub blocks: u64,
    pub ripostes: u64,
    pub resists: u64,
    pub healing: u64,
    pub overheal: u64,
    pub hps: f64,
    pub overheal_pct: f64,
    pub kills: u64,
    pub active: bool,
    pub players: Vec<PlayerStat>,
    pub timeline: Vec<TimelinePoint>,
    pub heal_timeline: Vec<TimelinePoint>,
    pub damage_types: Vec<TypeStat>,
    pub heal_spells: Vec<TypeStat>,
    pub taken_sources: Vec<TypeStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterPlayer {
    pub name: String,
    pub class_name: String,
    pub level: u32,
    pub group: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidRoster {
    pub captured_at: f64,
    pub players: Vec<RosterPlayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterState {
    pub character: Option<String>,
    pub server: Option<String>,
    pub stance: Option<String>,
    pub log_path: Option<String>,
    pub monitoring: bool,
    pub focus_primary: bool,
    pub min_fight_damage: u64,
    /// Live NPC fights (one row per mob, EQLogParser-style).
    pub active_fights: Vec<FightSummary>,
    /// Combined view of all live fights, or the single live fight.
    pub active_fight: Option<FightSummary>,
    pub recent_fights: Vec<FightSummary>,
    /// Latest /who all raid capture (Group: N lines).
    pub raid_roster: Option<RaidRoster>,
    pub recent_rosters: Vec<RaidRoster>,
    pub misc_log: Vec<MiscEvent>,
    pub spells_count: u64,
    pub spells_path: Option<String>,
}

#[derive(Debug, Default)]
struct PlayerAccum {
    damage: u64,
    hits: u64,
    crits: u64,
    max_hit: u64,
    attempts: u64,
    misses: u64,
    healing: u64,
    overheal: u64,
    healing_received: u64,
    abilities: HashMap<String, AbilityStat>,
    timeline: HashMap<u32, u64>,
    heal_timeline: HashMap<u32, u64>,
}

#[derive(Debug)]
struct Fight {
    id: u64,
    target: String,
    damage_to_target: u64,
    hits_to_target: u64,
    kills: u64,
    started_at: f64,
    last_hit_at: f64,
    ended_at: Option<f64>,
    players: HashMap<String, PlayerAccum>,
    timeline: HashMap<u32, u64>,
    heal_timeline: HashMap<u32, u64>,
    damage_types: HashMap<String, (u64, u64)>,
    heal_spells: HashMap<String, (u64, u64)>,
    max_hit: u64,
    max_hit_by: Option<String>,
    damage_taken: u64,
    taken_hits: u64,
    max_taken_hit: u64,
    taken_sources: HashMap<String, (u64, u64)>,
    attempts: u64,
    misses: u64,
    dodges: u64,
    parries: u64,
    blocks: u64,
    ripostes: u64,
    resists: u64,
    healing: u64,
    overheal: u64,
}

impl Fight {
    fn new(id: u64, target: String, now: f64) -> Self {
        Self {
            id,
            target,
            damage_to_target: 0,
            hits_to_target: 0,
            kills: 0,
            started_at: now,
            last_hit_at: now,
            ended_at: None,
            players: HashMap::new(),
            timeline: HashMap::new(),
            heal_timeline: HashMap::new(),
            damage_types: HashMap::new(),
            heal_spells: HashMap::new(),
            max_hit: 0,
            max_hit_by: None,
            damage_taken: 0,
            taken_hits: 0,
            max_taken_hit: 0,
            taken_sources: HashMap::new(),
            attempts: 0,
            misses: 0,
            dodges: 0,
            parries: 0,
            blocks: 0,
            ripostes: 0,
            resists: 0,
            healing: 0,
            overheal: 0,
        }
    }

    fn duration(&self, now: f64) -> f64 {
        let end = self.ended_at.unwrap_or(now);
        (end - self.started_at).max(1.0)
    }

    fn total_damage(&self) -> u64 {
        self.players.values().map(|p| p.damage).sum()
    }

    fn to_summary(&self, now: f64, active: bool, character: Option<&str>) -> FightSummary {
        let duration = self.duration(now);
        let total = self.total_damage();
        let total_hits: u64 = self.players.values().map(|p| p.hits).sum();
        let crits: u64 = self.players.values().map(|p| p.crits).sum();
        let attempts = self.attempts.max(total_hits);
        let misses = self.misses;

        let heal_total = self.healing;
        let mut players: Vec<PlayerStat> = self
            .players
            .iter()
            .map(|(name, accum)| player_stat(name, accum, duration, total, heal_total))
            .collect();

        if let Some(self_name) = character {
            let present = players.iter().any(|p| p.name.eq_ignore_ascii_case(self_name));
            if !present {
                players.push(PlayerStat {
                    name: self_name.to_string(),
                    damage: 0,
                    hits: 0,
                    crits: 0,
                    max_hit: 0,
                    dps: 0.0,
                    pct: 0.0,
                    attempts: 0,
                    misses: 0,
                    accuracy_pct: 0.0,
                    healing: 0,
                    overheal: 0,
                    hps: 0.0,
                    heal_pct: 0.0,
                    healing_received: 0,
                    abilities: Vec::new(),
                    timeline: Vec::new(),
                    heal_timeline: Vec::new(),
                });
            }
        }

        players.sort_by(|a, b| b.damage.cmp(&a.damage).then_with(|| a.name.cmp(&b.name)));

        let mut damage_types: Vec<TypeStat> = self
            .damage_types
            .iter()
            .map(|(name, (damage, hits))| TypeStat {
                name: name.clone(),
                damage: *damage,
                hits: *hits,
                pct: if total == 0 {
                    0.0
                } else {
                    (*damage as f64 / total as f64) * 100.0
                },
            })
            .collect();
        damage_types.sort_by(|a, b| b.damage.cmp(&a.damage));

        let mut heal_spells: Vec<TypeStat> = self
            .heal_spells
            .iter()
            .map(|(name, (amount, hits))| TypeStat {
                name: name.clone(),
                damage: *amount,
                hits: *hits,
                pct: if heal_total == 0 {
                    0.0
                } else {
                    (*amount as f64 / heal_total as f64) * 100.0
                },
            })
            .collect();
        heal_spells.sort_by(|a, b| b.damage.cmp(&a.damage));

        let taken_total = self.damage_taken;
        let mut taken_sources: Vec<TypeStat> = self
            .taken_sources
            .iter()
            .map(|(name, (damage, hits))| TypeStat {
                name: name.clone(),
                damage: *damage,
                hits: *hits,
                pct: if taken_total == 0 {
                    0.0
                } else {
                    (*damage as f64 / taken_total as f64) * 100.0
                },
            })
            .collect();
        taken_sources.sort_by(|a, b| b.damage.cmp(&a.damage));

        let timeline = map_to_timeline(&self.timeline);
        let heal_timeline = map_to_timeline(&self.heal_timeline);
        let peak_dps = peak_rolling_dps(&timeline, 3);
        let attempted_heal = self.healing + self.overheal;
        let targets = vec![TypeStat {
            name: self.target.clone(),
            damage: self.damage_to_target,
            hits: self.hits_to_target,
            pct: if total == 0 {
                0.0
            } else {
                (self.damage_to_target as f64 / total as f64) * 100.0
            },
        }];

        FightSummary {
            id: self.id,
            target: self.target.clone(),
            targets,
            started_at: self.started_at,
            ended_at: self.ended_at,
            duration_secs: duration,
            total_damage: total,
            total_dps: total as f64 / duration,
            peak_dps,
            total_hits,
            crits,
            crit_pct: if total_hits == 0 {
                0.0
            } else {
                (crits as f64 / total_hits as f64) * 100.0
            },
            max_hit: self.max_hit,
            max_hit_by: self.max_hit_by.clone(),
            damage_taken: self.damage_taken,
            taken_hits: self.taken_hits,
            dtps: self.damage_taken as f64 / duration,
            max_taken_hit: self.max_taken_hit,
            attempts,
            misses,
            accuracy_pct: if attempts == 0 {
                0.0
            } else {
                ((attempts - misses) as f64 / attempts as f64) * 100.0
            },
            dodges: self.dodges,
            parries: self.parries,
            blocks: self.blocks,
            ripostes: self.ripostes,
            resists: self.resists,
            healing: self.healing,
            overheal: self.overheal,
            hps: self.healing as f64 / duration,
            overheal_pct: if attempted_heal == 0 {
                0.0
            } else {
                (self.overheal as f64 / attempted_heal as f64) * 100.0
            },
            kills: self.kills,
            active,
            players,
            timeline,
            heal_timeline,
            damage_types,
            heal_spells,
            taken_sources,
        }
    }
}

fn player_stat(
    name: &str,
    accum: &PlayerAccum,
    duration: f64,
    total: u64,
    heal_total: u64,
) -> PlayerStat {
    let mut abilities: Vec<AbilityStat> = accum.abilities.values().cloned().collect();
    abilities.sort_by(|a, b| {
        b.damage
            .cmp(&a.damage)
            .then_with(|| b.healing.cmp(&a.healing))
    });
    let attempts = accum.attempts.max(accum.hits);
    PlayerStat {
        name: name.to_string(),
        damage: accum.damage,
        hits: accum.hits,
        crits: accum.crits,
        max_hit: accum.max_hit,
        dps: accum.damage as f64 / duration,
        pct: if total == 0 {
            0.0
        } else {
            (accum.damage as f64 / total as f64) * 100.0
        },
        attempts,
        misses: accum.misses,
        accuracy_pct: if attempts == 0 {
            0.0
        } else {
            ((attempts - accum.misses) as f64 / attempts as f64) * 100.0
        },
        healing: accum.healing,
        overheal: accum.overheal,
        hps: accum.healing as f64 / duration,
        heal_pct: if heal_total == 0 {
            0.0
        } else {
            (accum.healing as f64 / heal_total as f64) * 100.0
        },
        healing_received: accum.healing_received,
        abilities,
        timeline: map_to_timeline(&accum.timeline),
        heal_timeline: map_to_timeline(&accum.heal_timeline),
    }
}

#[derive(Debug, Default)]
pub struct FightTracker {
    next_id: u64,
    character: Option<String>,
    server: Option<String>,
    stance: Option<String>,
    log_path: Option<String>,
    monitoring: bool,
    focus_primary: bool,
    min_fight_damage: u64,
    active: HashMap<String, Fight>,
    recent: Vec<Fight>,
    /// Players captured since last raid-count flush.
    roster_building: HashMap<String, RosterPlayer>,
    raid_roster: Option<RaidRoster>,
    recent_rosters: Vec<RaidRoster>,
    misc_log: Vec<MiscEvent>,
    /// Pet display name → owner character name.
    pet_owners: HashMap<String, String>,
}

impl FightTracker {
    pub fn set_character(&mut self, name: Option<String>) {
        self.character = name;
    }

    pub fn set_server(&mut self, server: Option<String>) {
        self.server = server;
    }

    pub fn set_stance(&mut self, stance: Option<String>) {
        if stance.is_some() {
            self.stance = stance;
        }
    }

    pub fn set_log_path(&mut self, path: Option<String>) {
        self.log_path = path;
    }

    pub fn set_monitoring(&mut self, monitoring: bool) {
        self.monitoring = monitoring;
    }

    pub fn set_options(&mut self, focus_primary: bool, min_fight_damage: u64) {
        self.focus_primary = focus_primary;
        self.min_fight_damage = min_fight_damage;
    }

    pub fn reset_active(&mut self) {
        let now = event_time_or_wall(None);
        let fights = std::mem::take(&mut self.active);
        for (_, mut fight) in fights {
            fight.ended_at = Some(now);
            self.push_recent(fight);
        }
    }

    pub fn clear_all(&mut self) {
        self.active.clear();
        self.recent.clear();
        self.roster_building.clear();
        self.raid_roster = None;
        self.recent_rosters.clear();
        self.misc_log.clear();
        self.pet_owners.clear();
    }

    pub fn ingest(&mut self, event: CombatEvent) {
        let event_time = match &event {
            CombatEvent::Damage(d) => d.time_secs,
            CombatEvent::Death(d) => d.time_secs,
            CombatEvent::Avoid(a) => a.time_secs,
            CombatEvent::Resist(r) => r.time_secs,
            CombatEvent::Heal(h) => h.time_secs,
            CombatEvent::Stance(s) => s.time_secs,
            CombatEvent::Who(w) => w.time_secs,
            CombatEvent::RaidCount(r) => r.time_secs,
            CombatEvent::Misc(m) => m.time_secs,
        };
        let now = event_time_or_wall(event_time);
        self.close_if_idle(now);

        match event {
            CombatEvent::Damage(dmg) => self.on_damage(dmg, now),
            CombatEvent::Death(death) => self.on_death(death, now),
            CombatEvent::Avoid(avoid) => self.on_avoid(avoid, now),
            CombatEvent::Resist(resist) => self.on_resist(resist, now),
            CombatEvent::Heal(heal) => self.on_heal(heal, now),
            CombatEvent::Stance(stance) => self.stance = Some(stance.name),
            CombatEvent::Who(who) => self.on_who(who, now),
            CombatEvent::RaidCount(_) => self.flush_roster(now),
            CombatEvent::Misc(misc) => self.on_misc(misc),
        }
    }

    pub fn snapshot(&self) -> MeterState {
        let now = now_secs();
        let character = self.character.as_deref();
        let mut active: Vec<&Fight> = self.active.values().collect();
        active.sort_by(|a, b| b.last_hit_at.partial_cmp(&a.last_hit_at).unwrap());
        let active_fights: Vec<FightSummary> = active
            .into_iter()
            .map(|fight| fight.to_summary(fight.last_hit_at, true, character))
            .collect();
        let active_fight = self.combine_active(now);
        let recent_fights: Vec<FightSummary> = self
            .recent
            .iter()
            .rev()
            .take(20)
            .map(|f| {
                let end = f.ended_at.unwrap_or(f.last_hit_at);
                f.to_summary(end, false, character)
            })
            .collect();

        MeterState {
            character: self.character.clone(),
            server: self.server.clone(),
            stance: self.stance.clone(),
            log_path: self.log_path.clone(),
            monitoring: self.monitoring,
            focus_primary: self.focus_primary,
            min_fight_damage: self.min_fight_damage,
            active_fights,
            active_fight,
            recent_fights,
            raid_roster: self.raid_roster.clone(),
            recent_rosters: self.recent_rosters.iter().rev().take(8).cloned().collect(),
            misc_log: self.misc_log.iter().rev().take(80).cloned().collect(),
            spells_count: 0,
            spells_path: None,
        }
    }

    pub fn format_parse(&self, fight_id: Option<u64>) -> Option<String> {
        let summary = self.summary_for_id(fight_id)?;
        Some(format_fight_parse(&summary))
    }

    pub fn format_parse_ids(&self, fight_ids: &[u64]) -> Option<String> {
        let summary = self.combine_by_ids(fight_ids)?;
        Some(format_fight_parse(&summary))
    }

    pub fn combine_by_ids(&self, fight_ids: &[u64]) -> Option<FightSummary> {
        if fight_ids.is_empty() {
            return self.combine_active(now_secs());
        }
        let now = now_secs();
        let character = self.character.as_deref();
        let mut fights = Vec::new();
        for id in fight_ids {
            if *id == 0 {
                if let Some(combined) = self.combine_active(now) {
                    return Some(combined);
                }
                continue;
            }
            if let Some(fight) = self.active.values().find(|fight| fight.id == *id) {
                fights.push(fight.to_summary(now.max(fight.last_hit_at), true, character));
                continue;
            }
            if let Some(fight) = self.recent.iter().find(|fight| fight.id == *id) {
                fights.push(fight.to_summary(
                    fight.ended_at.unwrap_or(fight.last_hit_at),
                    false,
                    character,
                ));
            }
        }
        if fights.is_empty() {
            return None;
        }
        if fights.len() == 1 {
            return Some(fights.remove(0));
        }
        // Prefer chronological order for Combined title.
        fights.sort_by(|a, b| a.started_at.partial_cmp(&b.started_at).unwrap());
        Some(combine_summaries(&fights))
    }

    fn summary_for_id(&self, fight_id: Option<u64>) -> Option<FightSummary> {
        let now = now_secs();
        let character = self.character.as_deref();
        if let Some(id) = fight_id {
            if id == 0 {
                return self.combine_active(now);
            }
            return self
                .active
                .values()
                .find(|fight| fight.id == id)
                .map(|fight| fight.to_summary(now.max(fight.last_hit_at), true, character))
                .or_else(|| {
                    self.recent.iter().find(|f| f.id == id).map(|f| {
                        f.to_summary(f.ended_at.unwrap_or(f.last_hit_at), false, character)
                    })
                });
        }
        if !self.active.is_empty() {
            self.combine_active(now)
        } else {
            self.recent.last().map(|f| {
                f.to_summary(f.ended_at.unwrap_or(f.last_hit_at), false, character)
            })
        }
    }

    fn on_damage(&mut self, dmg: DamageEvent, now: f64) {
        if dmg.incoming {
            self.on_incoming(dmg, now);
            return;
        }

        let resolved = self.resolve_attacker(&dmg.attacker);
        let attacker = resolved.owner;
        let target = dmg.target.clone();
        let Some(fight) = self.fight_for(&target, now) else {
            return;
        };
        fight.last_hit_at = now;
        fight.damage_to_target += dmg.amount;
        fight.hits_to_target += 1;

        let sec = ((now - fight.started_at).floor() as i64).max(0) as u32;
        *fight.timeline.entry(sec).or_insert(0) += dmg.amount;

        let category = damage_category(&dmg);
        let entry = fight.damage_types.entry(category).or_insert((0, 0));
        entry.0 += dmg.amount;
        entry.1 += 1;

        let is_crit = is_crit_mod(&dmg);

        if dmg.amount > fight.max_hit {
            fight.max_hit = dmg.amount;
            fight.max_hit_by = Some(attacker.clone());
        }

        let base_ability = dmg
            .spell
            .clone()
            .unwrap_or_else(|| dmg.hit_type.clone());
        let ability_name = match &resolved.pet {
            Some(pet) => format!("Pet ({pet}): {base_ability}"),
            None => base_ability,
        };

        let player = fight.players.entry(attacker).or_default();
        player.damage += dmg.amount;
        player.hits += 1;
        player.attempts += 1;
        fight.attempts += 1;
        if is_crit {
            player.crits += 1;
        }
        if dmg.amount > player.max_hit {
            player.max_hit = dmg.amount;
        }
        *player.timeline.entry(sec).or_insert(0) += dmg.amount;

        let ability = player
            .abilities
            .entry(ability_name.clone())
            .or_insert(AbilityStat {
                name: ability_name,
                hits: 0,
                damage: 0,
                healing: 0,
            });
        ability.hits += 1;
        ability.damage += dmg.amount;
    }

    fn on_incoming(&mut self, dmg: DamageEvent, now: f64) {
        let source = self.resolve_attacker(&dmg.attacker).owner;
        if is_self_label(&source) || is_self_label(&dmg.attacker) {
            return;
        }
        let Some(fight) = self.fight_for(&source, now) else {
            return;
        };
        fight.last_hit_at = now;
        fight.damage_taken += dmg.amount;
        fight.taken_hits += 1;
        if dmg.amount > fight.max_taken_hit {
            fight.max_taken_hit = dmg.amount;
        }
        let entry = fight.taken_sources.entry(source).or_insert((0, 0));
        entry.0 += dmg.amount;
        entry.1 += 1;
    }

    fn on_avoid(&mut self, avoid: AvoidEvent, now: f64) {
        if avoid.incoming {
            let Some(fight) = self.fight_for(&avoid.attacker, now) else {
                return;
            };
            fight.last_hit_at = now;
            match avoid.outcome.as_str() {
                "miss" => {}
                "dodge" => fight.dodges += 1,
                "parry" => fight.parries += 1,
                "block" => fight.blocks += 1,
                "riposte" => fight.ripostes += 1,
                _ => {}
            }
            return;
        }

        let attacker = self.resolve_attacker(&avoid.attacker).owner;
        let Some(fight) = self.fight_for(&avoid.target, now) else {
            return;
        };
        fight.last_hit_at = now;
        fight.attempts += 1;
        fight.misses += 1;

        let player = fight.players.entry(attacker).or_default();
        player.attempts += 1;
        player.misses += 1;
    }

    fn on_resist(&mut self, resist: ResistEvent, now: f64) {
        let Some(fight) = self.fight_for(&resist.caster, now) else {
            return;
        };
        fight.last_hit_at = now;
        fight.resists += 1;
    }

    fn on_heal(&mut self, heal: HealEvent, now: f64) {
        // Attach heals to the active fight; don't open a new fight from heals alone.
        let healer = self.resolve_attacker(&heal.healer).owner;
        let target = self.resolve_attacker(&heal.target).owner;
        let Some(key) = self.most_recent_active_key() else {
            return;
        };
        let fight = self.active.get_mut(&key).expect("active fight");
        fight.last_hit_at = now;
        fight.healing += heal.amount;
        fight.overheal += heal.overheal;

        let sec = ((now - fight.started_at).floor() as i64).max(0) as u32;
        *fight.heal_timeline.entry(sec).or_insert(0) += heal.amount;

        let spell_entry = fight
            .heal_spells
            .entry(heal.spell.clone())
            .or_insert((0, 0));
        spell_entry.0 += heal.amount;
        spell_entry.1 += 1;

        let player = fight.players.entry(healer).or_default();
        player.healing += heal.amount;
        player.overheal += heal.overheal;
        *player.heal_timeline.entry(sec).or_insert(0) += heal.amount;

        let ability = player
            .abilities
            .entry(heal.spell.clone())
            .or_insert(AbilityStat {
                name: heal.spell.clone(),
                hits: 0,
                damage: 0,
                healing: 0,
            });
        ability.hits += 1;
        ability.healing += heal.amount;

        let receiver = fight.players.entry(target).or_default();
        receiver.healing_received += heal.amount;
    }

    fn on_death(&mut self, death: DeathEvent, now: f64) {
        if death.self_death {
            if let Some(key) = self.most_recent_active_key() {
                let fight = self.active.get_mut(&key).expect("active fight");
                fight.last_hit_at = now;
            }
            return;
        }

        let key = fight_key(&death.target);
        if let Some(mut fight) = self.active.remove(&key) {
            fight.kills += 1;
            fight.last_hit_at = now;
            fight.ended_at = Some(now);
            self.push_recent(fight);
        }
    }

    fn on_misc(&mut self, misc: MiscEvent) {
        self.misc_log.push(misc);
        if self.misc_log.len() > 200 {
            let overflow = self.misc_log.len() - 200;
            self.misc_log.drain(0..overflow);
        }
    }

    fn on_who(&mut self, who: WhoPlayerEvent, now: f64) {
        self.roster_building.insert(
            who.name.clone(),
            RosterPlayer {
                name: who.name,
                class_name: who.class_name,
                level: who.level,
                group: who.group,
            },
        );
        let mut players: Vec<RosterPlayer> = self.roster_building.values().cloned().collect();
        players.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
        self.raid_roster = Some(RaidRoster {
            captured_at: now,
            players,
        });
    }

    fn flush_roster(&mut self, now: f64) {
        if self.roster_building.is_empty() {
            return;
        }
        let mut players: Vec<RosterPlayer> =
            self.roster_building.drain().map(|(_, player)| player).collect();
        players.sort_by(|a, b| a.group.cmp(&b.group).then_with(|| a.name.cmp(&b.name)));
        let roster = RaidRoster {
            captured_at: now,
            players,
        };
        self.raid_roster = Some(roster.clone());
        self.recent_rosters.push(roster);
        if self.recent_rosters.len() > 12 {
            let overflow = self.recent_rosters.len() - 12;
            self.recent_rosters.drain(0..overflow);
        }
    }

    fn fight_for(&mut self, target: &str, now: f64) -> Option<&mut Fight> {
        if target.is_empty() || is_self_label(target) {
            return None;
        }
        // Always open/update a per-NPC fight so multi-mob pulls stay visible.
        // (focus_primary used to drop other targets — that hid adds until the
        // primary died.)
        let key = fight_key(target);
        if !self.active.contains_key(&key) {
            self.next_id += 1;
            self.active.insert(key.clone(), Fight::new(self.next_id, target.to_string(), now));
        }
        self.active.get_mut(&key)
    }

    fn most_recent_active_key(&self) -> Option<String> {
        self.active
            .iter()
            .max_by(|(_, a), (_, b)| a.last_hit_at.partial_cmp(&b.last_hit_at).unwrap())
            .map(|(key, _)| key.clone())
    }

    fn close_if_idle(&mut self, now: f64) {
        let keys: Vec<String> = self
            .active
            .iter()
            .filter(|(_, fight)| now - fight.last_hit_at >= IDLE_SECS)
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            if let Some(mut fight) = self.active.remove(&key) {
                fight.ended_at = Some(fight.last_hit_at);
                self.push_recent(fight);
            }
        }
    }

    fn combine_active(&self, _now: f64) -> Option<FightSummary> {
        let character = self.character.as_deref();
        let mut fights: Vec<&Fight> = self.active.values().collect();
        fights.sort_by(|a, b| a.started_at.partial_cmp(&b.started_at).unwrap());
        if fights.is_empty() {
            return None;
        }
        if fights.len() == 1 {
            let fight = fights[0];
            return Some(fight.to_summary(fight.last_hit_at, true, character));
        }
        let summaries: Vec<FightSummary> = fights
            .iter()
            .map(|fight| fight.to_summary(fight.last_hit_at, true, character))
            .collect();
        Some(combine_summaries(&summaries))
    }

    fn push_recent(&mut self, fight: Fight) {
        if self.min_fight_damage > 0 && fight.total_damage() < self.min_fight_damage {
            return;
        }
        self.recent.push(fight);
        if self.recent.len() > 40 {
            let overflow = self.recent.len() - 40;
            self.recent.drain(0..overflow);
        }
    }

    fn resolve_attacker(&mut self, raw: &str) -> ResolvedAttacker {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return ResolvedAttacker {
                owner: trimmed.to_string(),
                pet: None,
            };
        }

        if is_self_label(trimmed) {
            let owner = self
                .character
                .clone()
                .unwrap_or_else(|| "You".to_string());
            return ResolvedAttacker {
                owner,
                pet: None,
            };
        }

        // "Your pet" / "My pet" / "Your fire elemental" → you + pet label
        if let Some(rest) = strip_your_prefix(trimmed) {
            let owner = self
                .character
                .clone()
                .unwrap_or_else(|| "You".to_string());
            let pet = pet_display_name(rest);
            if !rest.is_empty() {
                self.note_pet(rest, &owner);
            }
            return ResolvedAttacker {
                owner,
                pet: Some(pet),
            };
        }

        // "Francis's pet" / "Francis`s pet" / "Francis pet"
        if let Some(owner) = owner_from_pet_label(trimmed) {
            return ResolvedAttacker {
                owner,
                pet: Some("Pet".to_string()),
            };
        }

        // Named pet we already mapped to an owner
        if let Some(owner) = self.pet_owners.get(&trimmed.to_ascii_lowercase()) {
            return ResolvedAttacker {
                owner: owner.clone(),
                pet: Some(trimmed.to_string()),
            };
        }

        ResolvedAttacker {
            owner: trimmed.to_string(),
            pet: None,
        }
    }

    /// Remember that a pet name belongs to an owner (usually you).
    pub fn note_pet(&mut self, pet_name: &str, owner: &str) {
        let key = pet_name.trim().to_ascii_lowercase();
        if key.is_empty() || key == "pet" {
            return;
        }
        self.pet_owners.insert(key, owner.to_string());
    }
}

struct ResolvedAttacker {
    owner: String,
    /// When set, damage is from a pet and should show in the owner's ability breakdown.
    pet: Option<String>,
}

fn pet_display_name(rest: &str) -> String {
    let trimmed = rest.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("pet") {
        return "Pet".to_string();
    }
    // "pet Fluffy" → Fluffy; otherwise keep the creature type/name.
    if let Some(name) = trimmed
        .strip_prefix("pet ")
        .or_else(|| trimmed.strip_prefix("Pet "))
    {
        let name = name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    trimmed.to_string()
}

fn is_self_label(name: &str) -> bool {
    name.eq_ignore_ascii_case("you")
        || name.eq_ignore_ascii_case("your")
        || name.eq_ignore_ascii_case("yourself")
        || name.eq_ignore_ascii_case("me")
}

fn strip_your_prefix(label: &str) -> Option<&str> {
    let bytes = label.as_bytes();
    if bytes.len() >= 5 && label[..5].eq_ignore_ascii_case("your ") {
        return Some(label[5..].trim());
    }
    if bytes.len() >= 3 && label[..3].eq_ignore_ascii_case("my ") {
        let rest = label[3..].trim();
        let rest_lower = rest.to_ascii_lowercase();
        if rest_lower == "pet" || rest_lower.starts_with("pet ") {
            return Some(rest);
        }
    }
    None
}

fn owner_from_pet_label(label: &str) -> Option<String> {
    let lower = label.to_ascii_lowercase();
    if let Some(owner) = lower.strip_suffix("'s pet") {
        let owner = owner.trim();
        if !owner.is_empty() {
            return Some(title_case_preserve(label, owner.len()));
        }
    }
    if let Some(owner) = lower.strip_suffix("`s pet") {
        let owner = owner.trim();
        if !owner.is_empty() {
            return Some(title_case_preserve(label, owner.len()));
        }
    }
    if let Some(owner) = lower.strip_suffix(" pet") {
        let owner = owner.trim();
        // Avoid treating "your pet" here (handled earlier) or NPC phrases.
        if !owner.is_empty() && !owner.contains(' ') {
            return Some(title_case_preserve(label, owner.len()));
        }
    }
    None
}

fn title_case_preserve(original: &str, owner_len: usize) -> String {
    original[..owner_len.min(original.len())].trim().to_string()
}

fn fight_key(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn combine_summaries(fights: &[FightSummary]) -> FightSummary {
    let first = &fights[0];
    let started_at = fights
        .iter()
        .map(|fight| fight.started_at)
        .fold(f64::INFINITY, f64::min);
    let last_hit_at = fights
        .iter()
        .map(|fight| fight.started_at + fight.duration_secs)
        .fold(f64::NEG_INFINITY, f64::max);
    let duration_secs = (last_hit_at - started_at).max(1.0);
    let total_damage: u64 = fights.iter().map(|fight| fight.total_damage).sum();
    let total_hits: u64 = fights.iter().map(|fight| fight.total_hits).sum();
    let healing: u64 = fights.iter().map(|fight| fight.healing).sum();

    let mut players = HashMap::<String, PlayerStat>::new();
    let mut damage_types = HashMap::<String, TypeStat>::new();
    let mut heal_spells = HashMap::<String, TypeStat>::new();
    let mut taken_sources = HashMap::<String, TypeStat>::new();
    let mut timeline = HashMap::<u32, u64>::new();
    let mut heal_timeline = HashMap::<u32, u64>::new();
    let mut targets = Vec::new();

    for fight in fights {
        for target in &fight.targets {
            targets.push(target.clone());
        }
        for player in &fight.players {
            merge_player(&mut players, player);
        }
        merge_type_stats(&mut damage_types, &fight.damage_types);
        merge_type_stats(&mut heal_spells, &fight.heal_spells);
        merge_type_stats(&mut taken_sources, &fight.taken_sources);
        merge_timeline(&mut timeline, &fight.timeline);
        merge_timeline(&mut heal_timeline, &fight.heal_timeline);
    }

    for target in &mut targets {
        target.pct = if total_damage == 0 {
            0.0
        } else {
            target.damage as f64 / total_damage as f64 * 100.0
        };
    }
    targets.sort_by(|a, b| b.damage.cmp(&a.damage).then_with(|| a.name.cmp(&b.name)));

    let mut players: Vec<PlayerStat> = players.into_values().collect();
    for player in &mut players {
        player.dps = player.damage as f64 / duration_secs;
        player.pct = if total_damage == 0 { 0.0 } else { player.damage as f64 / total_damage as f64 * 100.0 };
        player.hps = player.healing as f64 / duration_secs;
        player.heal_pct = if healing == 0 { 0.0 } else { player.healing as f64 / healing as f64 * 100.0 };
        player.accuracy_pct = if player.attempts == 0 { 0.0 } else { (player.attempts - player.misses) as f64 / player.attempts as f64 * 100.0 };
    }
    players.sort_by(|a, b| b.damage.cmp(&a.damage).then_with(|| a.name.cmp(&b.name)));

    let timeline = map_to_timeline(&timeline);
    let heal_timeline = map_to_timeline(&heal_timeline);
    let mut damage_types: Vec<TypeStat> = damage_types.into_values().collect();
    let mut heal_spells: Vec<TypeStat> = heal_spells.into_values().collect();
    let mut taken_sources: Vec<TypeStat> = taken_sources.into_values().collect();
    sort_type_stats(&mut damage_types);
    sort_type_stats(&mut heal_spells);
    sort_type_stats(&mut taken_sources);

    let damage_taken: u64 = fights.iter().map(|fight| fight.damage_taken).sum();
    let attempts: u64 = fights.iter().map(|fight| fight.attempts).sum();
    let misses: u64 = fights.iter().map(|fight| fight.misses).sum();
    let overheal: u64 = fights.iter().map(|fight| fight.overheal).sum();
    let crits: u64 = fights.iter().map(|fight| fight.crits).sum();
    let max_fight = fights.iter().max_by_key(|fight| fight.max_hit).unwrap();

    FightSummary {
        id: 0,
        target: format!("Combined ({}): {}", fights.len(), first.target),
        targets,
        started_at,
        ended_at: None,
        duration_secs,
        total_damage,
        total_dps: total_damage as f64 / duration_secs,
        peak_dps: peak_rolling_dps(&timeline, 3),
        total_hits,
        crits,
        crit_pct: if total_hits == 0 { 0.0 } else { crits as f64 / total_hits as f64 * 100.0 },
        max_hit: max_fight.max_hit,
        max_hit_by: max_fight.max_hit_by.clone(),
        damage_taken,
        taken_hits: fights.iter().map(|fight| fight.taken_hits).sum(),
        dtps: damage_taken as f64 / duration_secs,
        max_taken_hit: fights.iter().map(|fight| fight.max_taken_hit).max().unwrap_or(0),
        attempts,
        misses,
        accuracy_pct: if attempts == 0 { 0.0 } else { (attempts - misses) as f64 / attempts as f64 * 100.0 },
        dodges: fights.iter().map(|fight| fight.dodges).sum(),
        parries: fights.iter().map(|fight| fight.parries).sum(),
        blocks: fights.iter().map(|fight| fight.blocks).sum(),
        ripostes: fights.iter().map(|fight| fight.ripostes).sum(),
        resists: fights.iter().map(|fight| fight.resists).sum(),
        healing,
        overheal,
        hps: healing as f64 / duration_secs,
        overheal_pct: if healing + overheal == 0 { 0.0 } else { overheal as f64 / (healing + overheal) as f64 * 100.0 },
        kills: fights.iter().map(|fight| fight.kills).sum(),
        active: true,
        players,
        timeline,
        heal_timeline,
        damage_types,
        heal_spells,
        taken_sources,
    }
}

fn merge_player(players: &mut HashMap<String, PlayerStat>, incoming: &PlayerStat) {
    if !players.contains_key(&incoming.name) {
        players.insert(incoming.name.clone(), incoming.clone());
        return;
    }
    let player = players.get_mut(&incoming.name).expect("existing player");
    player.damage += incoming.damage;
    player.hits += incoming.hits;
    player.crits += incoming.crits;
    player.max_hit = player.max_hit.max(incoming.max_hit);
    player.attempts += incoming.attempts;
    player.misses += incoming.misses;
    player.healing += incoming.healing;
    player.overheal += incoming.overheal;
    player.healing_received += incoming.healing_received;
    merge_abilities(&mut player.abilities, &incoming.abilities);
    merge_player_timeline(&mut player.timeline, &incoming.timeline);
    merge_player_timeline(&mut player.heal_timeline, &incoming.heal_timeline);
}

fn merge_abilities(existing: &mut Vec<AbilityStat>, incoming: &[AbilityStat]) {
    for ability in incoming {
        if let Some(current) = existing.iter_mut().find(|item| item.name == ability.name) {
            current.hits += ability.hits;
            current.damage += ability.damage;
            current.healing += ability.healing;
        } else {
            existing.push(ability.clone());
        }
    }
}

fn merge_player_timeline(existing: &mut Vec<TimelinePoint>, incoming: &[TimelinePoint]) {
    let mut map = HashMap::new();
    merge_timeline(&mut map, existing);
    merge_timeline(&mut map, incoming);
    *existing = map_to_timeline(&map);
}

fn merge_type_stats(target: &mut HashMap<String, TypeStat>, incoming: &[TypeStat]) {
    for stat in incoming {
        let entry = target.entry(stat.name.clone()).or_insert(TypeStat {
            name: stat.name.clone(),
            damage: 0,
            hits: 0,
            pct: 0.0,
        });
        entry.damage += stat.damage;
        entry.hits += stat.hits;
    }
}

fn merge_timeline(target: &mut HashMap<u32, u64>, incoming: &[TimelinePoint]) {
    for point in incoming {
        *target.entry(point.sec).or_insert(0) += point.damage;
    }
}

fn sort_type_stats(stats: &mut [TypeStat]) {
    let total: u64 = stats.iter().map(|stat| stat.damage).sum();
    for stat in stats.iter_mut() {
        stat.pct = if total == 0 { 0.0 } else { stat.damage as f64 / total as f64 * 100.0 };
    }
    stats.sort_by(|a, b| b.damage.cmp(&a.damage).then_with(|| a.name.cmp(&b.name)));
}

pub fn format_fight_parse(fight: &FightSummary) -> String {
    let mut parts = Vec::new();
    parts.push(format!(
        "[EQL] {} {}",
        fight.target,
        format_duration(fight.duration_secs)
    ));

    for player in fight.players.iter().take(8) {
        parts.push(format!(
            "{} {} ({})",
            player.name,
            round_num(player.dps),
            compact_num(player.damage)
        ));
    }

    parts.push(format!(
        "Total {} ({} dps)",
        compact_num(fight.total_damage),
        round_num(fight.total_dps)
    ));

    if fight.damage_taken > 0 {
        parts.push(format!(
            "Taken {} ({} dtps)",
            compact_num(fight.damage_taken),
            round_num(fight.dtps)
        ));
    }

    if fight.attempts > 0 {
        parts.push(format!("Acc {:.0}%", fight.accuracy_pct));
    }

    if fight.healing > 0 {
        parts.push(format!(
            "Heal {} ({} hps)",
            compact_num(fight.healing),
            round_num(fight.hps)
        ));
    }

    parts.join(" | ")
}

fn format_duration(secs: f64) -> String {
    let total = secs.floor().max(0.0) as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

fn round_num(n: f64) -> String {
    format!("{}", n.round() as u64)
}

fn compact_num(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}m", n as f64 / 1_000_000.0)
    } else if n >= 10_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn is_crit_mod(dmg: &DamageEvent) -> bool {
    dmg.modifiers.iter().any(|m| {
        m.eq_ignore_ascii_case("Critical") || m.to_ascii_lowercase().contains("critical")
    })
}

fn damage_category(dmg: &DamageEvent) -> String {
    if dmg.hit_type == "dot" {
        return "DoT".to_string();
    }
    if dmg.spell.is_some() {
        return "Spell".to_string();
    }
    match dmg.hit_type.as_str() {
        "magic" | "fire" | "cold" | "poison" | "disease" | "chromatic" | "corruption"
        | "non-melee" => "Spell".to_string(),
        _ => "Melee".to_string(),
    }
}

fn map_to_timeline(map: &HashMap<u32, u64>) -> Vec<TimelinePoint> {
    let mut points: Vec<TimelinePoint> = map
        .iter()
        .map(|(sec, damage)| TimelinePoint {
            sec: *sec,
            damage: *damage,
        })
        .collect();
    points.sort_by_key(|p| p.sec);
    points
}

fn peak_rolling_dps(timeline: &[TimelinePoint], window: u32) -> f64 {
    if timeline.is_empty() {
        return 0.0;
    }
    let max_sec = timeline.last().map(|p| p.sec).unwrap_or(0);
    let mut by_sec = vec![0u64; (max_sec as usize) + 1];
    for point in timeline {
        if let Some(slot) = by_sec.get_mut(point.sec as usize) {
            *slot = point.damage;
        }
    }

    let window = window.max(1) as usize;
    let mut best = 0.0f64;
    let mut running = 0u64;
    for (i, value) in by_sec.iter().enumerate() {
        running += *value;
        if i >= window {
            running -= by_sec[i - window];
        }
        let span = (i + 1).min(window) as f64;
        let dps = running as f64 / span;
        if dps > best {
            best = dps;
        }
    }
    best
}

fn now_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn event_time_or_wall(event_time: Option<f64>) -> f64 {
    event_time.unwrap_or_else(now_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{CombatEvent, DamageEvent};

    fn hit(at: f64, target: &str, amount: u64) -> CombatEvent {
        CombatEvent::Damage(DamageEvent {
            timestamp: String::new(),
            time_secs: Some(at),
            incoming: false,
            attacker: "You".into(),
            target: target.into(),
            amount,
            hit_type: "hit".into(),
            spell: None,
            modifiers: Vec::new(),
        })
    }

    #[test]
    fn keeps_separate_fights_per_npc() {
        let mut tracker = FightTracker::default();
        tracker.set_character(Some("Kenkyo".into()));

        tracker.ingest(hit(100.0, "a skeleton", 50));
        tracker.ingest(hit(103.5, "a kor ghoul wizard", 80));
        tracker.ingest(hit(107.0, "a zombie", 60));

        let snap = tracker.snapshot();
        assert_eq!(snap.active_fights.len(), 3);
        let fight = snap.active_fight.expect("combined active fight");
        assert!(fight.target.starts_with("Combined (3):"));
        assert_eq!(fight.targets.len(), 3);
        assert!(fight.targets.iter().any(|t| t.name == "a kor ghoul wizard"));
        assert_eq!(snap.recent_fights.len(), 0);
        assert_eq!(fight.total_damage, 190);
    }

    #[test]
    fn multi_mob_even_when_focus_primary_flag_set() {
        let mut tracker = FightTracker::default();
        tracker.set_character(Some("Kenkyo".into()));
        tracker.set_options(true, 0);

        tracker.ingest(hit(100.0, "a skeleton", 50));
        tracker.ingest(hit(101.0, "a kor ghoul wizard", 80));
        tracker.ingest(hit(102.0, "a zombie", 60));

        let snap = tracker.snapshot();
        assert_eq!(snap.active_fights.len(), 3);
        assert_eq!(snap.active_fight.expect("combined").total_damage, 190);
    }

    #[test]
    fn closes_fight_on_npc_death() {
        let mut tracker = FightTracker::default();
        tracker.set_character(Some("Kenkyo".into()));

        tracker.ingest(hit(100.0, "a skeleton", 50));
        tracker.ingest(CombatEvent::Death(crate::parse::DeathEvent {
            timestamp: String::new(),
            time_secs: Some(101.0),
            target: "a skeleton".into(),
            killer: Some("Kenkyo".into()),
            self_death: false,
        }));
        tracker.ingest(hit(102.0, "a kor ghoul wizard", 80));

        let snap = tracker.snapshot();
        assert_eq!(snap.active_fights.len(), 1);
        let active = snap.active_fight.expect("wizard fight");
        assert_eq!(active.target, "a kor ghoul wizard");
        assert_eq!(snap.recent_fights.len(), 1);
        assert_eq!(snap.recent_fights[0].target, "a skeleton");
    }

    #[test]
    fn merges_you_dots_and_pet_onto_character() {
        let mut tracker = FightTracker::default();
        tracker.set_character(Some("Kenkyo".into()));

        tracker.ingest(hit(100.0, "a skeleton", 50));
        tracker.ingest(CombatEvent::Damage(DamageEvent {
            timestamp: String::new(),
            time_secs: Some(100.5),
            incoming: false,
            attacker: "You".into(),
            target: "a skeleton".into(),
            amount: 44,
            hit_type: "dot".into(),
            spell: Some("Blood Siphon Strike".into()),
            modifiers: Vec::new(),
        }));
        tracker.ingest(CombatEvent::Damage(DamageEvent {
            timestamp: String::new(),
            time_secs: Some(101.0),
            incoming: false,
            attacker: "Your pet".into(),
            target: "a skeleton".into(),
            amount: 30,
            hit_type: "hit".into(),
            spell: None,
            modifiers: Vec::new(),
        }));
        tracker.ingest(CombatEvent::Damage(DamageEvent {
            timestamp: String::new(),
            time_secs: Some(101.5),
            incoming: false,
            attacker: "Your fire elemental".into(),
            target: "a skeleton".into(),
            amount: 20,
            hit_type: "hit".into(),
            spell: None,
            modifiers: Vec::new(),
        }));

        let fight = tracker.snapshot().active_fight.expect("fight");
        assert_eq!(fight.players.len(), 1);
        assert_eq!(fight.players[0].name, "Kenkyo");
        assert_eq!(fight.players[0].damage, 144);
        assert!(!fight.players.iter().any(|p| p.name == "You"));
        assert!(!fight.players.iter().any(|p| p.name == "Yourself"));

        let abilities: Vec<&str> = fight.players[0]
            .abilities
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        assert!(abilities.iter().any(|n| *n == "hit"));
        assert!(abilities
            .iter()
            .any(|n| *n == "Blood Siphon Strike"));
        assert!(abilities.iter().any(|n| n.starts_with("Pet (Pet):")));
        assert!(abilities
            .iter()
            .any(|n| n.starts_with("Pet (fire elemental):")));
        let pet_total: u64 = fight.players[0]
            .abilities
            .iter()
            .filter(|a| a.name.starts_with("Pet ("))
            .map(|a| a.damage)
            .sum();
        assert_eq!(pet_total, 50);
    }

    #[test]
    fn ignores_self_hurt_cannibalize() {
        let mut tracker = FightTracker::default();
        tracker.set_character(Some("Kenkyo".into()));

        // Out of combat cannibalize / DS self tick must not open a fight.
        if let Some(event) = crate::parse::parse_line(
            "[Fri Jul 10 21:25:09 2026] You hurt yourself for 3 points.",
        ) {
            tracker.ingest(event);
        }
        let snap = tracker.snapshot();
        assert!(snap.active_fight.is_none());
        assert!(snap.active_fights.is_empty());
    }
}

