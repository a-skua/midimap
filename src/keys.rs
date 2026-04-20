use enigo::{Direction, Enigo, Key, Keyboard};

pub fn parse_combo(parts: &[String]) -> Result<Vec<Key>, String> {
    if parts.is_empty() {
        return Err("empty key combo".into());
    }
    parts.iter().map(|p| parse_key(p.trim())).collect()
}

fn parse_key(s: &str) -> Result<Key, String> {
    match s.to_lowercase().as_str() {
        "cmd" | "meta" | "super" | "win" => Ok(Key::Meta),
        "ctrl" | "control" => Ok(Key::Control),
        "alt" | "option" => Ok(Key::Alt),
        "shift" => Ok(Key::Shift),
        "return" | "enter" => Ok(Key::Return),
        "space" => Ok(Key::Space),
        "tab" => Ok(Key::Tab),
        "esc" | "escape" => Ok(Key::Escape),
        "delete" | "del" => Ok(Key::Delete),
        "backspace" => Ok(Key::Backspace),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" => Ok(Key::PageUp),
        "pagedown" => Ok(Key::PageDown),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        s if s.chars().count() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        s => Err(format!("Unknown key: '{}'", s)),
    }
}

pub fn trigger_combo(enigo: &mut Enigo, keys: &[Key]) {
    if keys.is_empty() {
        return;
    }
    let (modifiers, tail) = keys.split_at(keys.len() - 1);
    let main_key = tail[0];

    for &k in modifiers {
        let _ = enigo.key(k, Direction::Press);
    }
    let _ = enigo.key(main_key, Direction::Click);
    for &k in modifiers.iter().rev() {
        let _ = enigo.key(k, Direction::Release);
    }
}
