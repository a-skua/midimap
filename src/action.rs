use enigo::{Enigo, Key, Keyboard};
use std::process::Command as ShellCommand;
use std::thread;

use crate::keys::trigger_combo;

pub enum Action {
    Combo { label: String, keys: Vec<Key> },
    Text(String),
    Shell(String),
}

impl Action {
    pub fn label(&self) -> String {
        match self {
            Action::Combo { label, .. } => format!("Keys({})", label),
            Action::Text(s) => format!("Text({})", s),
            Action::Shell(s) => format!("Shell({})", s),
        }
    }

    pub fn run(&self, enigo: &mut Enigo) {
        match self {
            Action::Combo { keys, .. } => trigger_combo(enigo, keys),
            Action::Text(s) => {
                let _ = enigo.text(s);
            }
            Action::Shell(cmd) => spawn_shell(cmd.clone()),
        }
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
