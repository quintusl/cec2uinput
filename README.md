# cec2uinput

A Linux utility that bridges CEC (Consumer Electronics Control) remote control events to Linux uinput keyboard events. This allows you to control Linux applications using your TV remote control via HDMI-CEC.

## Features

- Receives CEC remote control events from HDMI devices
- Maps CEC button presses to configurable keyboard actions
- Uses Linux uinput to generate keyboard events
- Configurable key mappings via YAML configuration
- SystemD service integration for automatic startup
- Prevents duplicate key events by filtering key repeats

## Requirements

- Linux operating system
- HDMI-CEC compatible hardware (TV, receiver, etc.)
- libcec development libraries
- Root privileges (required for uinput device access)

## Installation

### Prerequisites

Install libcec development packages:

```bash
# Ubuntu/Debian
sudo apt-get install libcec-dev libcec6

# Fedora/RHEL
sudo dnf install libcec-devel libcec

# Arch Linux
sudo pacman -S libcec
```

### Build from Source

1. Clone the repository:
```bash
git clone <repository-url>
cd cec2uinput
```

2. Build the project:
```bash
cargo build --release
```

3. Install the binary:
```bash
sudo cp target/release/cec2uinput /usr/local/bin/
sudo chmod +x /usr/local/bin/cec2uinput
```

## Configuration

1. Copy the example configuration:
```bash
cp config.example.yaml config.yaml
```

2. Edit the configuration file:
```yaml
device_name: "cec2uinput"
vendor_id: 0x1234
product_id: 0x5678
mappings:
  up: "Up"
  down: "Down"
  left: "Left"
  right: "Right"
  select: "Return"
  f1: "F1"    # Blue button
  f2: "F2"    # Red button
  f3: "F3"    # Green button
  f4: "F4"    # Yellow button
```

### Configuration Options

- `device_name`: Virtual input device name (appears in `/proc/bus/input/devices`)
- `vendor_id`: USB vendor ID for the virtual device
- `product_id`: USB product ID for the virtual device
- `mappings`: Map CEC button names to keyboard actions

### Available CEC Button Names

- `up`, `down`, `left`, `right` - Navigation buttons
- `select` - Enter/OK button
- `f1` - Blue button
- `f2` - Red button
- `f3` - Green button
- `f4` - Yellow button

### Available Keyboard Actions

- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Numbers: `0`-`9`
- Function keys: `F1`-`F24`
- Special keys: `Return`, `Esc`, `Space`, `Tab`, `Home`, `End`, `PageUp`, `PageDown`

## Usage

### Manual Execution

Run the application manually (requires root):
```bash
sudo ./cec2uinput
```

### SystemD Service

1. Install the service file:
```bash
sudo cp cec2uinput.service /etc/systemd/system/
```

2. Enable and start the service:
```bash
sudo systemctl enable cec2uinput
sudo systemctl start cec2uinput
```

3. Check service status:
```bash
sudo systemctl status cec2uinput
```

4. View logs:
```bash
sudo journalctl -u cec2uinput -f
```

## Architecture

The application consists of two main components:

1. **CEC Event Handler** (`main.rs`):
   - Initializes libcec connection
   - Registers callback for CEC keypress events
   - Maps CEC button codes to action names
   - Filters duplicate key events

2. **UInput Device** (`linux.rs`):
   - Creates virtual input device using Linux uinput
   - Translates action names to keyboard events
   - Sends keyboard events to the system

## Troubleshooting

### Common Issues

1. **Permission Denied**:
   - Ensure you're running with root privileges
   - Check that your user has access to `/dev/uinput`

2. **CEC Device Not Found**:
   - Verify HDMI-CEC is enabled on your TV/receiver
   - Check that libcec can detect your CEC adapter
   - Test with `cec-client` command

3. **No Key Events**:
   - Check the configuration file mappings
   - Verify CEC events are being received (check logs)
   - Ensure the target application can receive keyboard events

### Debug Commands

Test CEC functionality:
```bash
# List CEC adapters
cec-client -l

# Monitor CEC events
cec-client -d 8
```

Check virtual input device:
```bash
# List input devices
cat /proc/bus/input/devices | grep -A 5 cec2uinput

# Monitor input events
sudo evtest
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Code Structure

- `src/main.rs` - Main application logic and CEC integration
- `src/linux.rs` - Linux-specific uinput implementation
- `config.yaml` - Runtime configuration
- `cec2uinput.service` - SystemD service definition

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the GNU General Public License v3.0. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [libcec-sys](https://crates.io/crates/libcec-sys) for CEC integration
- Uses [uinput](https://crates.io/crates/uinput) for Linux input event generation
- Powered by the [libcec](https://github.com/Pulse-Eight/libcec) library