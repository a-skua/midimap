use enigo::{Enigo, Settings};
use midir::{Ignore, MidiInput};
use std::sync::mpsc;

use crate::config::{Config, Mapping};

enum MidiEvent {
    NoteOn { channel: u8, note: u8 },
    ControlChange { channel: u8, cc: u8, value: u8 },
}

pub fn cmd_list() -> Result<(), Box<dyn std::error::Error>> {
    let mut midi_in = MidiInput::new("midimap")?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();

    if ports.is_empty() {
        println!("No MIDI input ports found.");
    } else {
        println!("Available MIDI input ports:");
        for (i, port) in ports.iter().enumerate() {
            println!("  [{}] {}", i, midi_in.port_name(port)?);
        }
    }
    Ok(())
}

pub fn cmd_run(config_path: Option<&str>, debug: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = Config::resolve_path(config_path)?;
    println!("Using config: {}", path.display());
    let config = Config::load(&path)?;
    let port_filter = config.port;

    let mappings: Vec<Mapping> = config
        .mappings
        .into_iter()
        .map(|m| m.resolve())
        .collect::<Result<_, String>>()?;

    let mut midi_in = MidiInput::new("midimap")?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();

    if ports.is_empty() {
        return Err("No MIDI input ports found.".into());
    }

    let port_idx = if let Some(filter) = &port_filter {
        ports
            .iter()
            .position(|p| {
                midi_in
                    .port_name(p)
                    .map(|n| n.contains(filter.as_str()))
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("No port matching '{}'", filter))?
    } else {
        0
    };

    let port = &ports[port_idx];
    let port_name = midi_in.port_name(port)?;
    println!("Listening on: {}", port_name);
    println!("Press Ctrl+C to stop.\n");

    let (tx, rx) = mpsc::channel::<MidiEvent>();

    let _conn = midi_in.connect(
        port,
        "midimap-input",
        move |_, msg, _| {
            if msg.len() < 2 {
                return;
            }
            let status = msg[0] & 0xF0;
            let channel = msg[0] & 0x0F;
            match status {
                // Note On with non-zero velocity
                0x90 if msg.len() >= 3 && msg[2] > 0 => {
                    let _ = tx.send(MidiEvent::NoteOn {
                        channel,
                        note: msg[1],
                    });
                }
                // Control Change
                0xB0 if msg.len() >= 3 => {
                    let _ = tx.send(MidiEvent::ControlChange {
                        channel,
                        cc: msg[1],
                        value: msg[2],
                    });
                }
                _ => {}
            }
        },
        (),
    )?;

    let mut enigo = Enigo::new(&Settings::default())?;

    for event in rx {
        match event {
            MidiEvent::NoteOn { channel, note } => {
                for m in &mappings {
                    if m.note != Some(note) {
                        continue;
                    }
                    // channel in MIDI is 0-indexed; config uses 1-indexed
                    if m.channel.is_some_and(|ch| ch != channel + 1) {
                        continue;
                    }
                    if debug {
                        let label = m.note_name.as_deref().unwrap_or("");
                        println!("note {} ({}) → {}", label, note, m.action.label());
                    }
                    m.action.run(&mut enigo);
                }
            }
            MidiEvent::ControlChange { channel, cc, value } => {
                for m in &mappings {
                    if m.cc != Some(cc) {
                        continue;
                    }
                    if m.channel.is_some_and(|ch| ch != channel + 1) {
                        continue;
                    }
                    if m.min_value.is_some_and(|min| value < min) {
                        continue;
                    }
                    if debug {
                        println!("cc {}={} → {}", cc, value, m.action.label());
                    }
                    m.action.run(&mut enigo);
                }
            }
        }
    }

    Ok(())
}
