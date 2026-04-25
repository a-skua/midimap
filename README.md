# midimap

Map MIDI events (notes, CC) to keyboard shortcuts on macOS and Linux.

Inspired by [midistroke](https://github.com/charlieroberts/midiStroke/),
rewritten in Rust with active dependencies.

## Requirements

- macOS or Linux (X11; Wayland is not supported by the underlying input library)
- Platform setup:
  - **macOS**: grant **Accessibility permission** — System Settings →
    Privacy & Security → Accessibility → enable your terminal or the
    midimap binary
  - **Linux (Debian/Ubuntu)**: install ALSA and libxdo dev headers
    ```bash
    sudo apt-get install libasound2-dev libxdo-dev
    ```

## Install

### Prebuilt binaries

Download from [Releases](https://github.com/a-skua/midimap/releases).
Builds are provided for:

- macOS (arm64)
- Linux (amd64, arm64)

### From source

Requires Rust (install via [rustup](https://rustup.rs)).

```bash
cargo install --path .
```

## Usage

```bash
# List available MIDI input ports
midimap list

# Run with a config file
midimap run                       # ./midimap.toml → ~/.config/midimap/config.toml
midimap run path/to/config.toml   # explicit path

# Print each triggered event (note/cc → action) for debugging
midimap run --debug
```

The config file is resolved in this order:

1. The path passed on the command line (if any)
2. `./midimap.toml` (current directory)
3. `~/.config/midimap/config.toml`

## Configuration

Copy `example.toml` to `midimap.toml` and edit:

```toml
# Optional: match port by name substring (uses first port if omitted)
port = "Arturia"

# Note On → key combo (LilyPond-style note names, scientific octave)
[[map]]
note = "c4"    # middle C (MIDI 60)
keys = ["cmd", "c"]

[[map]]
note = "cis4"  # C#4
keys = ["cmd", "v"]

# CC → key (with optional value threshold)
[[map]]
cc = 64        # sustain pedal
min_value = 64 # trigger on press only
keys = ["space"]

# Channel-specific (1–16; omit to match any channel)
[[map]]
note = "c4"
channel = 2
keys = ["cmd", "shift", "c"]

# Literal text input (any Unicode; use `text` instead of `keys`)
[[map]]
note = "a4"
text = "α"

# Shell command (executed via `sh -c`, non-blocking)
[[map]]
note = "b4"
sh = "say hello"      # macOS; on Linux try e.g. `notify-send hello`
```

Each mapping must specify exactly one of `keys`, `text`, or `sh`:

- `keys` — key combo as an array of modifiers plus a key, e.g. `["cmd", "c"]`
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

`keys` is an array of tokens — modifiers first, then the main key:

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
