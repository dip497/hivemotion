use std::collections::BTreeMap;
use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A point in film time: plain seconds, a beat name, or a beat name with an offset
/// (`drop`, `drop+0.25`, `board-0.05`). Beats are the only owner of time.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeRef {
    Seconds(f64),
    Beat { name: String, offset: f64 },
}

impl TimeRef {
    pub fn parse(text: &str) -> Result<TimeRef, String> {
        let text = text.trim();
        if let Ok(seconds) = text.parse::<f64>() {
            return Ok(TimeRef::Seconds(seconds));
        }
        let split = text
            .char_indices()
            .skip(1)
            .find(|(_, c)| *c == '+' || *c == '-')
            .map(|(i, _)| i);
        let (name, offset) = match split {
            Some(i) => {
                let offset: f64 = text[i..]
                    .trim()
                    .parse()
                    .map_err(|_| format!("'{text}' has an offset that is not a number"))?;
                (text[..i].trim(), offset)
            }
            None => (text, 0.0),
        };
        if !is_beat_name(name) {
            return Err(format!(
                "'{text}' is not a time: write seconds, a beat name, or a beat name with +/- seconds"
            ));
        }
        Ok(TimeRef::Beat {
            name: name.to_string(),
            offset,
        })
    }
}

pub fn is_beat_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

impl fmt::Display for TimeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeRef::Seconds(s) => write!(f, "{s}"),
            TimeRef::Beat { name, offset } if *offset == 0.0 => write!(f, "{name}"),
            TimeRef::Beat { name, offset } if *offset > 0.0 => write!(f, "{name}+{offset}"),
            TimeRef::Beat { name, offset } => write!(f, "{name}{offset}"),
        }
    }
}

impl Serialize for TimeRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            TimeRef::Seconds(v) => s.serialize_f64(*v),
            other => s.serialize_str(&other.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for TimeRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Number(f64),
            Text(String),
        }
        match Raw::deserialize(d)? {
            Raw::Number(v) => Ok(TimeRef::Seconds(v)),
            Raw::Text(t) => TimeRef::parse(&t).map_err(serde::de::Error::custom),
        }
    }
}

impl JsonSchema for TimeRef {
    fn schema_name() -> String {
        "TimeRef".into()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let value = serde_json::json!({
            "description": "Seconds, a beat name, or a beat name with an offset such as `drop+0.25`",
            "anyOf": [
                { "type": "number" },
                { "type": "string", "pattern": "^[a-z][a-z0-9_]*\\s*([+-]\\s*[0-9]+(\\.[0-9]+)?)?$" }
            ]
        });
        serde_json::from_value(value).expect("a valid schema")
    }
}

/// Named beats, in seconds of film time.
pub type Beats = BTreeMap<String, f64>;

pub fn resolve(beats: &Beats, time: &TimeRef) -> Result<f64, String> {
    match time {
        TimeRef::Seconds(s) => Ok(*s),
        TimeRef::Beat { name, offset } => match beats.get(name) {
            Some(at) => Ok(at + offset),
            None => {
                let known: Vec<&str> = beats.keys().map(String::as_str).collect();
                Err(format!(
                    "no beat named '{name}' (beats: {})",
                    known.join(", ")
                ))
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_negative_offset_moves_the_time_earlier() {
        let beats = Beats::from([("board".to_string(), 2.0)]);
        let at = resolve(&beats, &TimeRef::parse("board-0.05").unwrap()).unwrap();
        assert!((at - 1.95).abs() < 1e-9);
    }
}
