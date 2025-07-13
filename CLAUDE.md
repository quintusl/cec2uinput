# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

cec2uinput is a Linux-only Rust application that bridges CEC (Consumer Electronics Control) remote control events to uinput keyboard events. It listens for CEC events from HDMI devices and translates them to configurable keyboard actions.

## Architecture

The application follows a callback-driven architecture with channel-based communication:

- **main.rs**: Entry point that initializes CEC connection, sets up event handling, and processes incoming CEC events
- **linux.rs**: Linux-specific uinput device implementation for sending keyboard events
- **Config structure**: YAML-based configuration for device settings and key mappings
- **CEC Integration**: Uses cec-rs safe wrapper for libcec CEC device communication
- **Event Loop**: Processes CEC events via callbacks and maps them to keyboard actions via uinput

### Key Components

- CEC event callback system using safe Rust closures with cec-rs
- Channel-based communication (`std::sync::mpsc::channel`) between CEC callback and main event loop
- Configurable key mappings from CEC remote buttons to keyboard actions
- Platform-specific compilation (Linux only with `#[cfg(target_os = "linux")]`)
- Duplicate key filtering using `keypress.duration.as_millis() == 0`

## Configuration

- `config.yaml`: Main configuration file with device settings and key mappings
- `config.example.yaml`: Example configuration showing available mappings
- Device name, vendor ID, and product ID are configurable
- Key mappings translate CEC button names to action strings

### CEC Button Mapping

The application maps CEC user control codes to configurable actions:
- `CecUserControlCode::Up/Down/Left/Right` -> "up/down/left/right"
- `CecUserControlCode::Select` -> "select"
- `CecUserControlCode::F1Blue/F2Red/F3Green/F4Yellow` -> "f1/f2/f3/f4"

### UInput Action Mapping

Linux.rs maps action strings to keyboard keys:
- Numbers: "0"-"9" -> `keyboard::Key::_0` to `keyboard::Key::_9`
- Actions: "select"/"enter" -> `keyboard::Key::Enter`, "exit" -> `keyboard::Key::Esc`
- Function keys: "blue"/"red"/"green"/"yellow" -> `keyboard::Key::F1`-`F4`

## Build Commands

- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo test` - Run tests (if any)
- `cargo run` - Build and run the application (requires root for uinput)

## Dependencies

- **cec-rs**: Safe Rust wrapper for libcec CEC device communication
- **uinput**: Linux input device creation and event generation
- **serde/serde_yaml**: Configuration file parsing
- **anyhow**: Error handling and context

## Deployment

- SystemD service file provided as `cec2uinput.service`
- Service runs as root (required for uinput device access)
- Binary typically installed to `/usr/local/bin/cec2uinput`
- Configuration file should be in the working directory as `config.yaml`

## Development Notes

- Application requires root privileges for uinput device creation
- Only compiles on Linux targets (conditional compilation with `#[cfg(target_os = "linux")]`)
- CEC events are processed via callback system with improved error handling
- Uses cec-rs for safe wrapper around libcec with better Raspberry Pi compatibility
- Enhanced error messages for common Raspberry Pi CEC initialization issues in main.rs:55-67
- Error handling uses Result types throughout
- Troubleshooting script provided as `cec_troubleshoot.sh` for diagnosing CEC issues

## Common Development Tasks

When adding new CEC button mappings:
1. Add the `CecUserControlCode` variant to the match statement in main.rs:82-96
2. Add the corresponding action string mapping in linux.rs:22-47
3. Update `config.example.yaml` with the new mapping

When debugging CEC issues:
1. Run `./cec_troubleshoot.sh` to diagnose CEC hardware setup
2. Check CEC device detection with `cec-client -l`
3. Verify config.txt settings on Raspberry Pi
4. Ensure libcec development packages are installed

## Testing

- Run with `sudo cargo run` for manual testing (requires root)
- Use `evtest` to monitor generated input events
- Check CEC events with `cec-client -d 8` for debugging
- SystemD service logs available via `journalctl -u cec2uinput`