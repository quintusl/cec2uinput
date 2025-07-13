# cec2uinput

A Linux utility that bridges CEC (Consumer Electronics Control) remote control events to Linux uinput keyboard events. This allows you to control Linux applications using your TV remote control via HDMI-CEC.

## Features

- Receives CEC remote control events from HDMI devices
- Maps CEC button presses to configurable keyboard actions
- Uses Linux uinput to generate keyboard events
- Configurable key mappings via YAML configuration
- SystemD service integration for automatic startup
- Prevents duplicate key events by filtering key repeats
- Automatic detection of CEC adapter

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

### From Source

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

### Debian/Ubuntu

1. Build the package:
```bash
sudo apt-get install debhelper
dpkg-buildpackage -us -uc
```

2. Install the package:
```bash
sudo dpkg -i ../cec2uinput_*.deb
```

### Arch Linux

1. Build the package:
```bash
cd AUR
makepkg -si
```

## Configuration

1. Copy the example configuration:
```bash
cp config.yml /etc/cec2uinput/config.yml
```

2. Edit the configuration file:
```yaml
device_name: "cec2uinput"
vendor_id: 0x1234
product_id: 0x5678
physical_address: 0x1000  # HDMI port 1 (0x1000), port 2 (0x2000), etc.
cec_version: "1.4"        # CEC version: 1.3, 1.4, or 2.0
mappings:
  Up: "up"
  Down: "down"
  Left: "left"
  Right: "right"
  Select: "enter"
  Exit: "esc"
  F1Blue: "f1"
  F2Red: "f2"
  F3Green: "f3"
  F4Yellow: "f4"
  Number0: "0"
  Number1: "1"
  Number2: "2"
  Number3: "3"
  Number4: "4"
  Number5: "5"
  Number6: "6"
  Number7: "7"
  Number8: "8"
  Number9: "9"
```

### Configuration Options

- `device_name`: Virtual input device name (appears in `/proc/bus/input/devices`)
- `vendor_id`: USB vendor ID for the virtual device
- `product_id`: USB product ID for the virtual device
- `physical_address`: The physical address of the HDMI port. Default is `0x1000` (HDMI port 1).
- `cec_version`: The CEC version to use. Can be `1.3`, `1.4`, or `2.0`. Default is `1.4`.
- `mappings`: Map CEC button names to keyboard actions.

### Available CEC Button Names

A full list of available CEC button names can be found in the `src/main.rs` file.

### Available Keyboard Actions

A full list of available keyboard actions can be found in the `src/linux.rs` file.

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

### Common causes on Raspberry Pi CM5:

- Missing libcec development packages (try: `sudo apt-get install libcec-dev`)
- CEC hardware not properly detected - check `dmesg | grep cec`
- Driver conflicts (try: `sudo modprobe cec` or check `/dev/cec*`)
- Run `cec-client -l` to check available adapters
- For CM5 dual HDMI, try specifying port: `RPI:0` or `RPI:1`
- Ensure user has permission to access CEC device (add to `video` group)
- Check if another process is using the CEC adapter
- Make sure `hdmi_ignore_cec_init=1` is NOT set in `/boot/config.txt`
- Try `sudo systemctl stop cec` if cec service is running
- Verify physical address in config matches your HDMI setup
- Check CEC topology with `cec-ctl --show-topology`

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

- Built with [cec-rs](https://crates.io/crates/cec-rs) for CEC integration
- Uses [uinput](https://crates.io/crates/uinput) for Linux input event generation
- Powered by the [libcec](https://github.com/Pulse-Eight/libcec) library
