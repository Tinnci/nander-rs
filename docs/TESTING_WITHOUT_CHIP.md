# Quick Start: Testing Your CH341A Without a Flash Chip

## 🎯 Purpose

You've successfully connected your CH341A programmer but don't have a flash chip yet? No problem! This guide shows you how to test and verify your programmer is working correctly.

## ✓ What You Saw

```
[20:19:38] === Detected WCH Devices ===
[20:19:38]   ✓ 1A86:5512 - CH341A (Supported)
[20:19:38] ===========================
[20:19:38] Programmer connected
[20:19:38] Chip detection failed: Unsupported flash chip: JEDEC ID = FF FF FF
```

**This is NORMAL!** 
- `FF FF FF` means no chip is connected
- Your programmer is working correctly
- You just need a flash chip to program

## 🔨 How to Test Without a Flash Chip

### Method 1: Run Diagnostic Tests (Recommended)

```bash
cargo run -- diagnostic
```

or

```bash
nander diagnostic
```

**What it tests:**
- ✓ USB communication
- ✓ SPI bus control
- ✓ GPIO pins (LED blink test)
- ✓ JEDEC ID reading capability

**Example Output:**
```
=== CH341A Programmer Diagnostics ===

Test 1: Basic USB Communication
  ✓ USB communication OK

Test 2: SPI Bus Status
  SPI Response: FF FF FF
  ℹ No chip detected (all pins floating high)
  ℹ This is NORMAL if no flash chip is connected

Test 3: GPIO Control (LED Test)
  Testing GPIO toggling (if your board has an LED, it may blink)
  ✓ GPIO control OK

Test 4: JEDEC ID Detection
  JEDEC ID: FF FF FF
  ℹ No flash chip detected
  ℹ This is expected if nothing is connected to the SPI bus

  How to test with a real chip:
    1. Connect a SPI flash chip to the CH341A
    2. Common chips: W25Q64, GD25Q128, MX25L128, etc.
    3. Ensure correct wiring: CS, CLK, MISO, MOSI, VCC, GND
    4. Re-run this diagnostic

=== Diagnostics Complete ===
```

### Method 2: Interactive SPI Tester (Advanced)

```bash
cargo run -- diagnostic --interactive
```

or

```bash
nander test -i
```

**What it does:**
- Lets you send raw SPI commands
- Perfect for learning SPI protocol
- Test individual commands

**Example Session:**
```
=== Interactive SPI Command Tester ===
Send raw SPI commands to test the bus
Example: Write 0x9F, Read 3 bytes (JEDEC ID)
Type 'quit' to exit

Enter command (hex bytes, e.g., '9F 00 00 00'): 9F 00 00 00
  TX: 9F 00 00 00
  RX: FF FF FF FF

Enter command (hex bytes, e.g., '9F 00 00 00'): AB
  TX: AB
  RX: FF

Enter command (hex bytes, e.g., '9F 00 00 00'): quit
```

## 📚 Common SPI Commands to Try

Without a chip, you'll get `FF` responses, but you can verify the programmer is sending:

```bash
# JEDEC Read ID (0x9F)
9F 00 00 00

# Read Status Register (0x05) - generic
05 00

# Write Enable (0x06)
06

# Read Data (0x03 + address)
03 00 00 00 00 00 00 00
```

## 🔌 Next Steps: Connect a Real Flash Chip

### Recommended Chips for Testing

**SPI NOR Flash** (easiest to test):
- W25Q64 (8MB)
- W25Q128 (16MB)
- GD25Q128 (16MB)
- MX25L12833F (16MB)

**Where to Buy:**
- AliExpress: Search for "W25Q64 SOIC8" or "SPI Flash"
- eBay: "SPI NOR flash chip"
- Local electronics stores

### Wiring

Most CH341A programmers have a 25XX series socket:

```
CH341A Pin:  Flash Chip:
  CS    →    CS (Pin 1)
  MISO  →    DO (Pin 2)
  WP    →    WP (Pin 3)
  GND   →    GND (Pin 4)
  MOSI  →    DI (Pin 5)
  CLK   →    CLK (Pin 6)
  HOLD  →    HOLD (Pin 7)
  VCC   →    VCC (Pin 8)
```

### Test with Real Chip

1. Insert chip into socket (Pin 1 indicator)
2. Run again:
   ```bash
   cargo run -- info
   ```
3. You should see:
   ```
   Chip detected: W25Q64 (Winbond)
   Capacity: 8 MB
   Type: SPI NOR
   ```

## 💡 Troubleshooting

### All Tests Pass But Still Can't Detect Chip

**Check:**
1. Chip orientation (Pin 1 alignment)
2. Power supply (should be 3.3V, NOT 5V for most chips!)
3. Chip is not locked/write-protected
4. Connections are clean and secure

### SPI Bus Shows "00 00 00"

**Possible Issues:**
- Short circuit
- Wrong voltage
- Chip is damaged
- Power supply problem

### GPIO Test Doesn't Blink LED

**Normal!** Not all CH341A boards have visible LEDs
- Some boards have LEDs on different pins
- Some have no LEDs at all
- If other tests pass, this is fine

## 🎓 Learning Resources

Want to learn more about SPI Flash?

- [SPI Flash Commands](https://www.winbond.com/resource-files/w25q128jv%20revf%2003272018%20plus.pdf) - W25Q128 datasheet
- [CH341A Protocol](https://github.com/McMCCRU/SNANDer) - Original SNANDer project
- Run `nander list` - See all supported chips

## ✅ Summary

Your programmer is working if:
- ✓ Diagnostic tests pass
- ✓ USB communication OK
- ✓ Gets `FF FF FF` response (no chip)

Ready to program when:
- You connect a real flash chip
- Run `nander info` successfully
- Can read/write/erase the chip

Happy hacking! 🚀
