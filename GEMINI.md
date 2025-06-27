# macOS Support

This project now supports macOS. To build and run on macOS, you need to install `libcec` via Homebrew:

```bash
brew install libcec
```

The `vendor_id` and `product_id` for your HID device (keyboard) need to be configured in `config.yaml`. You can find these values using your system's device information tools.
