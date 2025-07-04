# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

cec2uinput is a Linux-only Rust application that bridges CEC (Consumer Electronics Control) remote control events to uinput keyboard events. It listens for CEC events from HDMI devices and translates them to configurable keyboard actions.

## Architecture

- **main.rs**: Entry point that initializes CEC connection, sets up event handling, and processes incoming CEC events
- **linux.rs**: Linux-specific uinput device implementation for sending keyboard events
- **Config structure**: YAML-based configuration for device settings and key mappings
- **CEC Integration**: Uses libcec-sys bindings for CEC device communication
- **Event Loop**: Processes CEC events and maps them to keyboard actions via uinput

## Key Components

- CEC event callback system using unsafe extern "C" functions
- Channel-based communication between CEC callback and main event loop
- Configurable key mappings from CEC remote buttons to keyboard actions
- Platform-specific compilation (Linux only)

## Configuration

- `config.yaml`: Main configuration file with device settings and key mappings
- `config.example.yaml`: Example configuration showing available mappings
- Device name, vendor ID, and product ID are configurable
- Key mappings translate CEC button names to action strings

## Build Commands

- `cargo build` - Build the project
- `cargo build --release` - Build optimized release version
- `cargo test` - Run tests (if any)
- `cargo run` - Build and run the application

## Dependencies

- libcec-sys: CEC device communication
- uinput: Linux input device creation
- serde/serde_yaml: Configuration file parsing
- anyhow: Error handling

## Deployment

- SystemD service file provided as `cec2uinput.service`
- Service runs as root (required for uinput device access)
- Binary typically installed to `/usr/local/bin/cec2uinput`

## Development Notes

- Application requires root privileges for uinput device creation
- Only compiles on Linux targets (conditional compilation with `#[cfg(target_os = "linux")]`)
- CEC events are processed in a continuous loop
- Error handling uses Result types throughout