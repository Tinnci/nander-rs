# WCH Device Support and Troubleshooting

## Overview

`nander-rs` now includes intelligent recognition for WinChipHead (WCH) / QinHeng Electronics USB devices. This guide helps you understand device compatibility and troubleshoot connection issues.

## Supported Devices

### ✓ Fully Supported

| Device | VID:PID | Description | Speed |
|--------|---------|-------------|-------|
| **CH341A** | 1A86:5512 | USB to SPI/I2C Bridge (Programmer Mode) | ~6 MHz SPI |

## WCH Device Recognition

### CH341 Series

#### CH341A (PID 0x5512) ✓ **Supported**
- **Full Name**: CH341A USB to SPI/I2C Bridge
- **Mode**: SPI/I2C Programmer Mode
- **Status**: Fully supported for flash programming

#### CH341 (PID 0x5523) ⚠ **Wrong Mode**
- **Full Name**: CH341 USB to Serial Adapter
- **Mode**: UART/Serial Mode
- **Status**: Device detected but in incompatible mode

**How to Fix:**
1. **Check Hardware Jumper**: Most CH341A programmer boards have a jumper or switch
   - Look for labels like "SPI/UART", "0/1", or "MODE"
   - Set to SPI/I2C mode (usually position "1" or marked as "SPI")
2. **Replug USB**: After changing the jumper, unplug and replug the USB connector
3. **Verify**: The device should now appear as PID 0x5512

**Common Jumper Locations:**
- Between the USB connector and the IC
- Near the ZIF socket
- On the back of the PCB

#### CH340 (PID 0x7523) ℹ **Related Device**
- **Full Name**: CH340 USB to Serial Adapter
- **Status**: Not a flash programmer, serial adapter only
- **Note**: You need a CH341A (0x5512), not CH340

### CH347 Series (Planned Support)

#### CH347 (PID 0x55DB) ◐ **Planned**
- **Full Name**: CH347 USB to UART/I2C/SPI Bridge (High-Speed)
- **Speed**: Up to 60 MHz SPI (10x faster than CH341A)
- **Status**: Support planned for future releases
- **Note**: Star our GitHub repo to get notified!

#### CH347F (PID 0x55DD) ℹ **Related Device**
- **Full Name**: CH347F USB to JTAG/SWD/UART Bridge
- **Primary Use**: JTAG/SWD debugging
- **Note**: Use CH347 (0x55DB) for flash programming

### CH348 Series

#### CH348 (PID 0x55D2, 0x55D3) ℹ **Related Device**
- **Full Name**: CH348 USB to Multi-Serial Port
- **Status**: Multi-port serial adapter, not a flash programmer

### CH9102/CH9103 Series

#### CH9102 (PID 0x55D4) ℹ **Related Device**
- **Full Name**: CH9102 USB to Serial (Type-C)
- **Status**: Modern Type-C serial adapter, not a flash programmer

## Diagnostic Tools

### Command Line

Run with debug logging to see detailed device information:

```bash
# Windows
$env:RUST_LOG="debug"
cargo run -- gui

# Linux/macOS
RUST_LOG=debug cargo run -- gui
```

### What You'll See

**Correct Device (PID 0x5512):**
```
[DEBUG] Scanning 12 USB devices...
[DEBUG] Found 1 WCH device(s):
[DEBUG]   ✓ 1A86:5512 - CH341A (Supported)
[DEBUG]     This is the correct mode for flash programming!
[DEBUG] ✓ Using: CH341A
```

**Wrong Mode (PID 0x5523):**
```
[DEBUG] Scanning 12 USB devices...
[DEBUG] Found 1 WCH device(s):
[DEBUG]   ⚠ 1A86:5523 - CH341 (Wrong Mode)
[DEBUG]     ⚠ Device is in UART mode, not SPI mode.
[DEBUG]     Solutions:
[DEBUG]     1. Check hardware jumper/switch on your CH341A board (look for SPI/UART or 0/1)
[DEBUG]     2. Some boards require shorting specific pins to enter SPI mode
[DEBUG]     3. Unplug device, change jumper, then replug USB
[DEBUG]     4. After switching, the device should appear as PID 0x5512
[ERROR] WCH device(s) found, but none in supported mode.
```

## GUI Mode

When using the GUI (`nander gui`):

1. Click **Connect**
2. Open the **Logs** section
3. Look for "=== Detected WCH Devices ===" section
4. Device information will be displayed with status indicators:
   - `✓` = Supported and ready
   - `⚠` = Wrong mode (fix needed)
   - `ℹ` = Related device (not compatible)
   - `◐` = Planned support
   - `?` = Unknown device

## Troubleshooting

### "No programmer detected"

**Possible Causes:**
1. Device not connected
2. Device in wrong mode (check jumper)
3. USB port issue
4. Driver issue (Windows)

**Solutions:**
1. Check USB cable connection
2. Try different USB port
3. Check device mode (see jumper instructions above)
4. On Windows: Check Device Manager for driver issues

### "Device found but can't connect"

**Possible Causes:**
1. Device in use by another program
2. Permission issues (Linux)
3. Driver conflict (Windows)

**Solutions:**
1. Close other programs using the device
2. (Linux) Add udev rules or run with `sudo` temporarily
3. (Windows) Uninstall conflicting drivers in Device Manager

### Linux udev Rules

Create `/etc/udev/rules.d/99-ch341a.rules`:

```
# CH341A SPI Programmer
SUBSYSTEM=="usb", ATTR{idVendor}=="1a86", ATTR{idProduct}=="5512", MODE="0666"
```

Reload rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

## Windows Driver Issues

### Error: "Device driver is 'CH341_A64', not WinUSB"

**Cause**: The CH341A is using the manufacturer's default driver which is incompatible with `nusb` (used by nander-rs).

**Solution**: Replace with WinUSB driver using Zadig

#### Step-by-Step Guide:

1. **Download Zadig**
   - Visit: https://zadig.akeo.ie/
   - Download the latest version
   - No installation needed, just run the `.exe`

2. **Prepare Zadig**
   - Right-click `zadig.exe` and select "Run as Administrator"
   - Go to `Options` menu → Check "List All Devices"

3. **Select Your Device**
   - In the dropdown at the top, find:
     - `USB-SERIAL CH341A` or
     - `Interface 0 (Interface 0)` or
     - Something containing "CH341"
   - Make sure VID/PID shows `1A86 5512`

4. **Check Current Driver**
   - The large box on the left shows current driver
   - You'll likely see: `CH341_A64`, `CH341SER`, or similar

5. **Select WinUSB**
   - Click the up/down arrows in the middle
   - Select **WinUSB** from the list (NOT libusb!)

6. **Install**
   - Click the big button: "Replace Driver" or "Install Driver"
   - Wait for the process to complete (usually 30-60 seconds)
   - You should see "Driver Installation Success"

7. **Verify**
   - Unplug and replug your CH341A device
   - Run `nander-rs` again
   - It should now connect successfully!

#### Important Notes:

✅ **This is safe**: You're only changing the driver, not modifying hardware
✅ **Reversible**: You can use Zadig again to switch back to the original driver anytime
✅ **Won't break other tools**: Other CH341 software may still work, or you can switch drivers when needed

❌ **Don't use libusb-win32**: Always select **WinUSB** for modern Windows systems

### Visual Guide

```
Zadig Window:
┌─────────────────────────────────────────────┐
│ Select Device: [USB-SERIAL CH341A         ▼]│
│                                              │
│ USB ID: 1A86 5512                            │
│                                              │
│ ┌──────────┐         ┌──────────┐          │
│ │CH341_A64 │  ═══►   │ WinUSB   │          │
│ └──────────┘         └──────────┘          │
│   Current              Target               │
│                                              │
│       [Replace Driver]                      │
└─────────────────────────────────────────────┘
```

## Reporting Issues

If you encounter an unknown WCH device (PID not listed), please report:

1. **Device Model**: Full model number from the chip or documentation
2. **VID:PID**: From Device Manager or `lsusb`
3. **Use Case**: What you're trying to do
4. **GitHub Issue**: https://github.com/YOUR_REPO/nander-rs/issues

Include the debug log output for fastest support!

## Future Plans

We're planning to add support for:

- **CH347 (0x55DB)**: High-speed USB-SPI bridge (up to 60 MHz)
- **CH342/CH343**: Modern USB-UART bridges with SPI capabilities
- **Automatic Mode Switching**: Software-triggered mode changes (if supported by hardware)

Star our repository to stay updated on new device support!
