use super::LineData;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// You assume a berserker stance.
static STANCE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^You assume an? (?P<name>.+?) stance\.$").expect("stance regex")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StanceEvent {
    pub timestamp: String,
    pub time_secs: Option<f64>,
    pub name: String,
}

pub fn parse_stance_line(data: &LineData) -> Option<StanceEvent> {
    let caps = STANCE_RE.captures(data.action.as_str())?;
    Some(StanceEvent {
        timestamp: data.timestamp.clone(),
        time_secs: data.time_secs,
        name: title_case(caps["name"].trim()),
    })
}

fn title_case(raw: &str) -> String {
    raw.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out = first.to_uppercase().collect::<String>();
                    out.push_str(&chars.as_str().to_ascii_lowercase());
                    out
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Scan log text newest-first for the latest stance.
pub fn detect_stance_from_text(text: &str) -> Option<String> {
    for line in text.lines().rev() {
        let Some(data) = super::split_log_line(line) else {
            continue;
        };
        if let Some(event) = parse_stance_line(&data) {
            return Some(event.name);
        }
    }
    None
}
