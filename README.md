# midimap

Map MIDI events (notes, CC) to keyboard shortcuts on macOS.

Inspired by [midistroke](https://github.com/charlieroberts/midiStroke/), rewritten in Rust with active dependencies.

## Requirements

- macOS
- Rust (install via [rustup](https://rustup.rs))
- **Accessibility permission** — System Settings → Privacy & Security → Accessibility → enable your terminal or the midimap binary

## Install

```bash
cargo install --path .
```

## Usage

```bash
# List available MIDI input ports
midimap list

# Run with a config file (default: midimap.toml)
midimap run
midimap run path/to/config.toml
```

## Configuration

Copy `example.toml` to `midimap.toml` and edit:

```toml
# Optional: match port by name substring (uses first port if omitted)
port = "Arturia"

# Note On → key combo
[[map]]
note = 60      # C4
keys = "cmd+c"

[[map]]
note = 61      # C#4
keys = "cmd+v"

# CC → key (with optional value threshold)
[[map]]
cc = 64        # sustain pedal
min_value = 64 # trigger on press only
keys = "space"

# Channel-specific (1–16; omit to match any channel)
[[map]]
note = 60
channel = 2
keys = "cmd+shift+c"
```

### Key syntax

Modifiers and keys are joined with `+`:

| Token | Key |
|---|---|
| `cmd` / `meta` | ⌘ Command |
| `ctrl` / `control` | ⌃ Control |
| `alt` / `option` | ⌥ Option |
| `shift` | ⇧ Shift |
| `f1`–`f12` | Function keys |
| `return` / `enter` | Return |
| `space`, `tab`, `esc` | Special keys |
| `up`, `down`, `left`, `right` | Arrow keys |
| `home`, `end`, `pageup`, `pagedown` | Navigation |
| any single character | `a`–`z`, `0`–`9`, etc. |

## License

MIT
