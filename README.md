# midimap

Map MIDI events (notes, CC) to keyboard shortcuts on macOS.

Inspired by [midistroke](https://github.com/charlieroberts/midiStroke/),
rewritten in Rust with active dependencies.

## Requirements

- macOS
- Rust (install via [rustup](https://rustup.rs))
- **Accessibility permission** — System Settings → Privacy & Security →
  Accessibility → enable your terminal or the midimap binary

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

# Note On → key combo (LilyPond-style note names, scientific octave)
[[map]]
note = "c4"    # middle C (MIDI 60)
keys = "cmd+c"

[[map]]
note = "cis4"  # C#4
keys = "cmd+v"

# CC → key (with optional value threshold)
[[map]]
cc = 64        # sustain pedal
min_value = 64 # trigger on press only
keys = "space"

# Channel-specific (1–16; omit to match any channel)
[[map]]
note = "c4"
channel = 2
keys = "cmd+shift+c"

# Literal text input (any Unicode; use `text` instead of `keys`)
[[map]]
note = "a4"
text = "α"

# Shell command (executed via `sh -c`, non-blocking)
[[map]]
note = "b4"
sh = "say hello"
```

Each mapping must specify exactly one of `keys`, `text`, or `sh`:

- `keys` — key combo (modifiers + a key), e.g. `"cmd+c"`
- `text` — type literal text; supports arbitrary Unicode
  (`"α"`, `"∀"`, `"こんにちは"`). Non-ASCII characters only
  work through `text`, not `keys`.
- `sh` — run a shell command via `sh -c`. Spawned on a background
  thread so the MIDI loop stays responsive.

### Note name syntax

Notes follow [LilyPond](https://lilypond.org/)-style spelling with a
scientific-pitch octave number (`c4` = middle C = MIDI 60).

| Form      | Meaning        | Example              |
| --------- | -------------- | -------------------- |
| `c`–`b`   | Natural        | `c4` → 60, `a4` → 69 |
| `…is`     | Sharp (♯)      | `cis4` → 61          |
| `…es`     | Flat (♭)       | `ees4` → 63          |
| `…isis`   | Double sharp   | `cisis4` → 62        |
| `…eses`   | Double flat    | `deses4` → 60        |
| `…<n>`    | Octave (any ℤ) | `c-1` → 0            |

### Key syntax

Modifiers and keys are joined with `+`:

| Token                               | Key                    |
| ----------------------------------- | ---------------------- |
| `cmd` / `meta`                      | ⌘ Command              |
| `ctrl` / `control`                  | ⌃ Control              |
| `alt` / `option`                    | ⌥ Option               |
| `shift`                             | ⇧ Shift                |
| `f1`–`f12`                          | Function keys          |
| `return` / `enter`                  | Return                 |
| `space`, `tab`, `esc`               | Special keys           |
| `up`, `down`, `left`, `right`       | Arrow keys             |
| `home`, `end`, `pageup`, `pagedown` | Navigation             |
| any single character                | `a`–`z`, `0`–`9`, etc. |

## License

MIT
