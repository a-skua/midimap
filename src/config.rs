use serde::Deserialize;
use std::fs;

use crate::action::Action;
use crate::keys::parse_combo;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub port: Option<String>,
    #[serde(default, rename = "map")]
    pub mappings: Vec<MappingConfig>,
}

#[derive(Debug, Deserialize)]
pub struct MappingConfig {
    /// LilyPond-style note name with scientific octave, e.g. "c4", "cis4", "ces4", "ees3".
    pub note: Option<String>,
    pub cc: Option<u8>,
    /// MIDI channel 1-16; omit to match any channel
    pub channel: Option<u8>,
    /// For CC: only trigger when value >= min_value
    pub min_value: Option<u8>,
    /// Key combo as an array, e.g. ["cmd", "c"], ["ctrl", "shift", "z"], ["f5"]
    pub keys: Option<Vec<String>>,
    /// Literal text to type (supports arbitrary Unicode, e.g. "α", "こんにちは")
    pub text: Option<String>,
    /// Shell command to run, executed via `sh -c`
    pub sh: Option<String>,
}

pub struct Mapping {
    pub note: Option<u8>,
    pub note_name: Option<String>,
    pub cc: Option<u8>,
    pub channel: Option<u8>,
    pub min_value: Option<u8>,
    pub action: Action,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Cannot read '{}': {}", path, e))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Invalid config: {}", e))?;
        Ok(config)
    }
}

impl MappingConfig {
    pub fn resolve(self) -> Result<Mapping, String> {
        let action = match (self.keys, self.text, self.sh) {
            (Some(key_parts), None, None) => {
                let label = key_parts.join("+");
                let keys = parse_combo(&key_parts)
                    .map_err(|e| format!("Invalid keys '{}': {}", label, e))?;
                Action::Combo { label, keys }
            }
            (None, Some(text), None) => Action::Text(text),
            (None, None, Some(sh)) => Action::Shell(sh),
            (None, None, None) => return Err("mapping must specify 'keys', 'text', or 'sh'".into()),
            _ => {
                return Err(
                    "mapping must specify exactly one of 'keys', 'text', or 'sh'".into(),
                )
            }
        };
        let note = match &self.note {
            Some(s) => Some(parse_note(s).map_err(|e| format!("Invalid note: {}", e))?),
            None => None,
        };
        Ok(Mapping {
            note,
            note_name: self.note,
            cc: self.cc,
            channel: self.channel,
            min_value: self.min_value,
            action,
        })
    }
}

/// Parse a LilyPond-style note name (with scientific octave number) to a MIDI note number.
///
/// Examples: `c4` → 60, `cis4` → 61, `ces4` → 59, `ees3` → 51, `c-1` → 0.
///
/// Syntax: `<letter><accidental?><octave>`
/// - letter: one of `c d e f g a b` (case-insensitive)
/// - accidental: `is` (♯), `es` (♭), `isis` (𝄪), `eses` (𝄫), or omitted
/// - octave: signed integer (scientific pitch notation; `c4` = middle C = MIDI 60)
pub fn parse_note(s: &str) -> Result<u8, String> {
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    if bytes.is_empty() {
        return Err("empty note name".into());
    }
    let pitch = match bytes[0] {
        b'c' => 0,
        b'd' => 2,
        b'e' => 4,
        b'f' => 5,
        b'g' => 7,
        b'a' => 9,
        b'b' => 11,
        c => return Err(format!("invalid note letter '{}' in '{}'", c as char, s)),
    };
    let rest = &lower[1..];
    let (accidental, rest) = if let Some(r) = rest.strip_prefix("isis") {
        (2i32, r)
    } else if let Some(r) = rest.strip_prefix("eses") {
        (-2i32, r)
    } else if let Some(r) = rest.strip_prefix("is") {
        (1i32, r)
    } else if let Some(r) = rest.strip_prefix("es") {
        (-1i32, r)
    } else {
        (0i32, rest)
    };
    let octave: i32 = rest
        .parse()
        .map_err(|_| format!("invalid octave in note '{}'", s))?;
    let midi = (octave + 1) * 12 + pitch + accidental;
    if !(0..=127).contains(&midi) {
        return Err(format!(
            "note '{}' yields MIDI {} which is out of range 0-127",
            s, midi
        ));
    }
    Ok(midi as u8)
}

#[cfg(test)]
mod tests {
    use super::parse_note;

    #[test]
    fn naturals() {
        assert_eq!(parse_note("c4").unwrap(), 60);
        assert_eq!(parse_note("a4").unwrap(), 69);
        assert_eq!(parse_note("c-1").unwrap(), 0);
        assert_eq!(parse_note("g9").unwrap(), 127);
    }

    #[test]
    fn accidentals() {
        assert_eq!(parse_note("cis4").unwrap(), 61);
        assert_eq!(parse_note("ces4").unwrap(), 59);
        assert_eq!(parse_note("ees4").unwrap(), 63);
        assert_eq!(parse_note("fis5").unwrap(), 78);
        assert_eq!(parse_note("bes3").unwrap(), 58);
        assert_eq!(parse_note("cisis4").unwrap(), 62);
        assert_eq!(parse_note("deses4").unwrap(), 60);
    }

    #[test]
    fn case_insensitive_and_trim() {
        assert_eq!(parse_note("C4").unwrap(), 60);
        assert_eq!(parse_note("  Cis4  ").unwrap(), 61);
    }

    #[test]
    fn errors() {
        assert!(parse_note("").is_err());
        assert!(parse_note("h4").is_err());
        assert!(parse_note("c").is_err());
        assert!(parse_note("cx4").is_err());
        assert!(parse_note("c10").is_err()); // MIDI 132, out of range
        assert!(parse_note("c-2").is_err()); // MIDI -12, out of range
    }
}
