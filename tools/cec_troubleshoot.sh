#!/bin/bash
# CEC Troubleshooting Script for Raspberry Pi

echo "=== CEC Troubleshooting on Raspberry Pi ==="
echo

echo "1. Checking for CEC devices:"
if command -v cec-client &> /dev/null; then
    cec-client -l
else
    echo "cec-client not found. Install with: sudo apt-get install cec-utils"
fi
echo

echo "2. Checking for CEC kernel modules:"
lsmod | grep -i cec || echo "No CEC modules loaded"
echo

echo "3. Checking /dev/cec* devices:"
ls -la /dev/cec* 2>/dev/null || echo "No /dev/cec* devices found"
echo

echo "4. Checking video core CEC:"
ls -la /dev/vchiq* 2>/dev/null || echo "No /dev/vchiq* devices found"
echo

echo "5. GPU memory split (should be >= 64MB for CEC):"
vcgencmd get_mem gpu 2>/dev/null || echo "vcgencmd not available"
echo

echo "6. Checking config.txt for CEC settings:"
if [ -f /boot/config.txt ]; then
    grep -i cec /boot/config.txt || echo "No CEC settings found in /boot/config.txt"
elif [ -f /boot/firmware/config.txt ]; then
    grep -i cec /boot/firmware/config.txt || echo "No CEC settings found in /boot/firmware/config.txt"
else
    echo "config.txt not found in standard locations"
fi
echo

echo "7. Testing CEC scan (requires root):"
if [ "$EUID" -eq 0 ]; then
    if command -v cec-client &> /dev/null; then
        timeout 5 cec-client -s || echo "CEC scan failed or timed out"
    else
        echo "cec-client not available"
    fi
else
    echo "Run as root to perform CEC scan: sudo $0"
fi
echo

echo "=== Troubleshooting Complete ==="
echo
echo "Common fixes for Raspberry Pi CEC issues:"
echo "1. Install CEC packages: sudo apt-get install cec-utils libcec4-dev"
echo "2. Enable CEC in config.txt: hdmi_ignore_cec=0"
echo "3. Load CEC module: sudo modprobe cec"
echo "4. Check GPU memory split: Add 'gpu_mem=64' to config.txt"
echo "5. For newer kernels, use cec-ctl instead of cec-client"