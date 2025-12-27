# Quick Start: Fixing CH341A Driver on Windows

## 🚨 Problem

You see this error:
```
✓ 1A86:5512 - CH341A (Supported)
Connection failed: USB error: Device driver is "CH341_A64", not WinUSB
```

## ✅ Solution (5 minutes)

### What You Need
- **Zadig** - A free Windows USB driver installer
- **5 minutes** - That's all!

### Step 1: Download Zadig
1. Go to: **https://zadig.akeo.ie/**
2. Click "Zadig 2.x" download link
3. Save the file (no installation needed)

### Step 2: Run Zadig as Administrator
1. Right-click `zadig.exe`
2. Select **"Run as administrator"**
3. Click "Yes" if Windows asks for permission

### Step 3: Enable "List All Devices"
1. Click **Options** menu at the top
2. Check ✓ **"List All Devices"**

### Step 4: Select Your CH341A
In the dropdown box at the top, find:
- **"USB-SERIAL CH341A"** or
- **"Interface 0"** or
- Something with "CH341"

Make sure it shows: `USB ID: 1A86 5512`

### Step 5: Change Driver
You'll see two boxes with an arrow between them:

```
┌────────────┐        ┌──────────┐
│ CH341_A64  │  ===>  │ WinUSB   │
└────────────┘        └──────────┘
   Current              Select this!
```

1. The **LEFT** box shows current driver (e.g., "CH341_A64")
2. Click the **UP/DOWN ARROWS** in the middle
3. Select **"WinUSB"** (NOT "libusb-win32"!)

### Step 6: Install
1. Click the big button: **"Replace Driver"**
2. Wait 30-60 seconds
3. You'll see "Driver Installation: SUCCESS ✓"

### Step 7: Test
1. **Unplug** your CH341A from USB
2. **Replug** it
3. Run `nander-rs gui` again
4. Click **"Connect"**
5. **Success!** 🎉

## 💡 FAQ

**Q: Will this break my other CH341 software?**
A: Maybe, but you can easily switch back using Zadig again.

**Q: Is this permanent?**
A: No! You can restore the original driver anytime with Zadig.

**Q: Do I need to do this every time?**
A: No, only once. Windows remembers the driver.

**Q: What if I selected the wrong device?**
A: Just run Zadig again and fix it. No harm done.

**Q: Why WinUSB and not libusb-win32?**
A: WinUSB is the modern, native Windows driver. libusb-win32 is outdated.

## 🔄 How to Restore Original Driver

If you need the original CH341 driver back:

1. Run Zadig as Administrator again
2. Select your CH341A device
3. Change driver back to "CH341_A64" or "CH341SER"
4. Click "Replace Driver"

## 🎯 Visual Guide

```
Before (Won't Work):
Device Manager → Ports (COM & LPT)
  └─ USB-SERIAL CH341 (COM3)  [Driver: CH341_A64]

After (Works!):
Device Manager → Universal Serial Bus devices
  └─ USB-SERIAL CH341A  [Driver: WinUSB]
```

## 🆘 Still Having Issues?

Check the full documentation:
- `docs/WCH_DEVICES.md` - Complete troubleshooting guide
- GitHub Issues - Report problems and get help

## 📝 Technical Details

**Why is this needed?**
- `nander-rs` uses `nusb`, a pure Rust USB library
- `nusb` requires WinUSB driver on Windows
- The manufacturer's CH341 driver doesn't expose the necessary interface
- Zadig replaces the driver to enable proper USB communication

**Is this safe?**
- ✅ YES! You're only changing software, not hardware
- ✅ Reversible anytime
- ✅ Used by thousands of developers worldwide
- ✅ Recommended by most USB development tools
