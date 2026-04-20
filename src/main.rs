mod config;
mod keys;

use clap::{Parser, Subcommand};
use config::{parse_note, Config};
use enigo::{Enigo, Key, Keyboard, Settings};
use keys::{parse_combo, trigger_combo};
use midir::{Ignore, MidiInput};
use std::process::Command as ShellCommand;
use std::sync::mpsc;
use std::thread;

#[derive(Parser)]
#[command(name = "midimap", about = "Map MIDI events to keyboard shortcuts")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available MIDI input ports
    List,
    /// Run the MIDI mapper with a config file
    Run {
        #[arg(default_value = "midimap.toml")]
        config: String,
    },
}

enum MidiEvent {
    NoteOn { channel: u8, note: u8 },
    ControlChange { channel: u8, cc: u8, value: u8 },
}

enum Action {
    Combo { label: String, keys: Vec<Key> },
    Text(String),
    Shell(String),
}

struct Mapping {
    note: Option<u8>,
    note_name: Option<String>,
    cc: Option<u8>,
    channel: Option<u8>,
    min_value: Option<u8>,
    action: Action,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::List => cmd_list(),
        Command::Run { config } => cmd_run(&config),
    }
}

fn action_label(action: &Action) -> &str {
    match action {
        Action::Combo { label, .. } => label,
        Action::Text(s) | Action::Shell(s) => s,
    }
}

fn run_action(enigo: &mut Enigo, action: &Action) {
    match action {
        Action::Combo { keys, .. } => trigger_combo(enigo, keys),
        Action::Text(s) => {
            let _ = enigo.text(s);
        }
        Action::Shell(cmd) => spawn_shell(cmd.clone()),
    }
}

fn spawn_shell(cmd: String) {
    thread::spawn(
        move || match ShellCommand::new("sh").arg("-c").arg(&cmd).spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => eprintln!("failed to spawn '{}': {}", cmd, e),
        },
    );
}

fn cmd_list() -> Result<(), Box<dyn std::error::Error>> {
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

fn cmd_run(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(config_path)?;
    let port_filter = config.port;

    let mappings: Vec<Mapping> = config
        .mappings
        .into_iter()
        .map(|m| {
            let action = match (m.keys, m.text, m.sh) {
                (Some(keys_str), None, None) => {
                    let keys = parse_combo(&keys_str)
                        .map_err(|e| format!("Invalid keys '{}': {}", keys_str, e))?;
                    Action::Combo {
                        label: keys_str,
                        keys,
                    }
                }
                (None, Some(text), None) => Action::Text(text),
                (None, None, Some(sh)) => Action::Shell(sh),
                (None, None, None) => {
                    return Err("mapping must specify 'keys', 'text', or 'sh'".into())
                }
                _ => {
                    return Err(
                        "mapping must specify exactly one of 'keys', 'text', or 'sh'".into(),
                    )
                }
            };
            let note = match &m.note {
                Some(s) => Some(parse_note(s).map_err(|e| format!("Invalid note: {}", e))?),
                None => None,
            };
            Ok(Mapping {
                note,
                note_name: m.note,
                cc: m.cc,
                channel: m.channel,
                min_value: m.min_value,
                action,
            })
        })
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
                    let label = m.note_name.as_deref().unwrap_or("");
                    println!("note {} ({}) → {}", label, note, action_label(&m.action));
                    run_action(&mut enigo, &m.action);
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
                    println!("cc {}={} → {}", cc, value, action_label(&m.action));
                    run_action(&mut enigo, &m.action);
                }
            }
        }
    }

    Ok(())
}
