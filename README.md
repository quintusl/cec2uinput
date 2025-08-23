# cec2uinput

A Linux utility that bridges HDMI-CEC (Consumer Electronics Control) remote control events to Linux uinput input events. It maps CEC remote buttons to configurable keyboard and mouse actions so you can control Linux applications with a TV remote.

## Features

- Receive CEC remote control events from HDMI devices
- Map CEC button presses to configurable keyboard and mouse actions
- Generate virtual keyboard and mouse events via Linux uinput
- Configurable mappings in `config/config.yml` (YAML)
- Systemd service integration for automatic startup
- Filters duplicate key events (ignore CEC key repeat frames)
- Automatic detection of CEC adapters and multiple fallback ports (including Raspberry Pi ports)

## Requirements

- Linux operating system (the binary only runs on Linux)
- HDMI-CEC compatible hardware and libcec development libraries
- Root privileges (or uinput access) to create the virtual input device

## Installation

### Prerequisites

Install `libcec` development packages for your distribution (examples):

```bash
# Ubuntu / Debian
sudo apt-get install libcec-dev libcec6

# Fedora / RHEL
sudo dnf install libcec-devel libcec

# Arch
sudo pacman -S libcec
```

### From source

```bash
git clone <repository-url>
cd cec2uinput
cargo build --release
sudo cp target/release/cec2uinput /usr/local/bin/
sudo chmod +x /usr/local/bin/cec2uinput
```

Packaging hints (Debian / Arch) remain available in the repository (see `debian/` and `AUR/`).

## Configuration

Copy and edit the example `config/config.yml` to `/etc/cec2uinput/config.yml` (or provide `--config` on the command line):

```bash
cp config/config.yml /etc/cec2uinput/config.yml
```

Basic example configuration snippet (more examples in `config/config.yml`):

```yaml
device_name: "CM5-CEC-Bridge"
cec_version: "1.4"
log_level: "info"
mappings:
  Up: "up"
  Down: "down"
  Left: "left"
  Right: "right"
  Select: "enter"
  Exit: "esc"
  Number0: "0"
  Number1: "1"
```

### Mouse support and mapping tokens

This version adds support for a virtual mouse device. You can map CEC buttons to mouse movements and clicks. Supported mouse mapping tokens (examples):

- `mouse_right` — move cursor right
- `mouse_left`  — move cursor left
- `mouse_up`    — move cursor up
- `mouse_down`  — move cursor down
- `mouse_click_left` (aliases: `mouse_left_click`, `mouse_lclick`) — left button click
- `mouse_click_right` (aliases: `mouse_right_click`, `mouse_rclick`) — right button click

Movement behaviour:

- Repeated mapped movement events accelerate using exponential steps: [1, 10, 50, 100, 500] pixels.
- Each direction (x+/x-/y+/y-) has its own counter and the counter resets after 500 milliseconds of idle mouse activity.

Example mapping that maps the CEC Right button to mouse movement right and Select to left click:

```yaml
mappings:
  Right: "mouse_right"
  Select: "mouse_click_left"
```

### Configuration options

- `device_name`: Virtual input device name (shown in `/proc/bus/input/devices`)
- `cec_version`: `1.3`, `1.4`, or `2.0` (default `1.4`)
- `mappings`: Map CEC button names (see `src/main.rs`) to actions (keyboard or mouse tokens supported)

## Usage

### Command line arguments

The binary uses `clap` for comprehensive argument parsing with detailed help. Run `cec2uinput --help` for full documentation.

**Available options:**

- `-c, --config <FILE>` — Path to the YAML configuration file. If omitted, uses `config.yml` from the current working directory.
- `-l, --log-level <LEVEL>` — Set logging verbosity: `error`, `warn`, `info` (default), `debug`, or `trace`. Overrides config file setting.
- `-q, --quiet` — Suppress all console output. Useful for running as daemon/service. Overrides any log level settings.
- `-h, --help` — Show comprehensive help with examples and exit.
- `-V, --version` — Show version information and exit.

**Examples:**

```bash
# Use default config.yml with info logging
sudo cec2uinput

# Use custom configuration file
sudo cec2uinput -c /etc/cec2uinput/config.yml

# Enable debug logging for troubleshooting
sudo cec2uinput -l debug

# Run silently (no console output)
sudo cec2uinput -q

# Combine options: error-only logging with custom config
sudo cec2uinput -l error -c /path/to/custom.yml
```

### Manual run

Run the binary (requires root privileges for uinput device access):

```bash
# Basic run with default config.yml
sudo cec2uinput

# With custom config and debug logging
sudo cec2uinput -c /etc/cec2uinput/config.yml -l debug
```

### Systemd

```bash
sudo cp config/cec2uinput.service /etc/systemd/system/cec2uinput.service
sudo systemctl enable --now cec2uinput
sudo journalctl -u cec2uinput -f
```

## Architecture

- `src/main.rs` — handles CEC connection (cec-rs), receives keypress callbacks and maps CEC codes to action names.
- `src/linux.rs` — builds a uinput virtual device and translates action names to keyboard and mouse events. This file contains the mapping table for keyboard tokens and the mouse handling logic (exponential movement, click events).

## Troubleshooting

- Permission errors: ensure the process can create / access the uinput device (run as root or give uinput access).
- No CEC events: verify HDMI-CEC is enabled on the TV and that libcec detects the adapter (`cec-client -l`).
- No keyboard/mouse events: check mappings in `config/config.yml` and watch the logs for messages about unknown actions.
- Debug issues with increased logging: run with `-l debug` or `-l trace` for detailed diagnostic output.

Debug commands:

```bash
# List CEC adapters
cec-client -l

# Monitor CEC events
cec-client -d 8

# List input devices and look for the virtual device
cat /proc/bus/input/devices | grep -A 5 cec2uinput

# Monitor events (choose appropriate device number)
sudo evtest
```

## Development

Build and test with cargo:

```bash
cargo build
cargo test
```

Code layout:

- `src/main.rs` — main application and CEC mapping table
- `src/linux.rs` — uinput device implementation (keyboard and mouse handling)
- `config/config.yml` — example configuration and mapping examples

## Contributing

Contributions are welcome — fork, branch, add tests, and open a PR.

## License

This project is licensed under the GNU General Public License v3.0. See `LICENSE`.

## Acknowledgements

- Built with `cec-rs` for CEC integration
- Uses `uinput` for Linux input event generation
- Powered by `libcec` (Pulse-Eight)
