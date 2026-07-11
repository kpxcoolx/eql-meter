use crate::parse::CombatEvent;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Optional EverQuest `spells_us.txt` catalog (caret-delimited).
/// Used to turn numeric spell IDs in log lines into readable names.
#[derive(Debug, Default, Clone)]
pub struct SpellCatalog {
    pub path: Option<String>,
    by_id: HashMap<u32, String>,
}

impl SpellCatalog {
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("read spells file: {e}"))?;
        Self::from_text(&text, Some(path.display().to_string()))
    }

    pub fn from_text(text: &str, path: Option<String>) -> Result<Self, String> {
        let mut by_id = HashMap::new();

        for line in text.lines() {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.split('^');
            let Some(id_raw) = parts.next() else {
                continue;
            };
            let Some(name) = parts.next() else {
                continue;
            };
            let Ok(id) = id_raw.trim().parse::<u32>() else {
                continue;
            };
            let name = name.trim();
            if name.is_empty() {
                continue;
            }
            by_id.insert(id, name.to_string());
        }

        if by_id.is_empty() {
            return Err("No ability names found — expected a caret-delimited EQ spell file".to_string());
        }

        Ok(Self { path, by_id })
    }

    pub fn resolve(&self, label: &str) -> Option<&str> {
        let trimmed = label.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(id) = trimmed.parse::<u32>() {
            return self.by_id.get(&id).map(|s| s.as_str());
        }
        let digits = trimmed
            .strip_prefix('#')
            .or_else(|| trimmed.strip_prefix("Spell "))
            .or_else(|| trimmed.strip_prefix("spell "))
            .unwrap_or(trimmed);
        if let Ok(id) = digits.trim().parse::<u32>() {
            return self.by_id.get(&id).map(|s| s.as_str());
        }
        None
    }

    pub fn enrich_event(&self, event: CombatEvent) -> CombatEvent {
        if self.is_empty() {
            return event;
        }
        match event {
            CombatEvent::Damage(mut dmg) => {
                if let Some(ref spell) = dmg.spell {
                    if let Some(name) = self.resolve(spell) {
                        dmg.spell = Some(name.to_string());
                    }
                }
                CombatEvent::Damage(dmg)
            }
            CombatEvent::Heal(mut heal) => {
                if let Some(name) = self.resolve(&heal.spell) {
                    heal.spell = name.to_string();
                }
                CombatEvent::Heal(heal)
            }
            CombatEvent::Resist(mut resist) => {
                if let Some(name) = self.resolve(&resist.spell) {
                    resist.spell = name.to_string();
                }
                CombatEvent::Resist(resist)
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_caret_delimited_spells() {
        let text = "100^Complete Heal^COMPLETE_HEAL^0\n200^Flame Lick^FLAME_LICK^0\n";
        let catalog = SpellCatalog::from_text(text, None).unwrap();
        assert_eq!(catalog.len(), 2);
        assert_eq!(catalog.resolve("100"), Some("Complete Heal"));
        assert_eq!(catalog.resolve("#200"), Some("Flame Lick"));
        assert_eq!(catalog.resolve("Spell 200"), Some("Flame Lick"));
    }
}
