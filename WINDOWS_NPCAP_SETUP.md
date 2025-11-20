# Windows Npcap Setup for Network Capture

## Problem
If you see a linker error like:
```
error: linking with `x86_64-w64-mingw32-gcc` failed
-lwpcap
```

This means Npcap is not installed or the linker can't find it.

## Solution

### Step 1: Install Npcap

1. **Download Npcap** from: https://nmap.org/npcap/
   - Choose the latest stable version
   - Download the installer (`.exe` file)

2. **Install Npcap**:
   - Run the installer as Administrator
   - **IMPORTANT**: Check the box for **"Install Npcap in WinPcap API-compatible Mode"**
   - This ensures compatibility with the `pcap` Rust crate
   - Complete the installation

3. **Restart your computer** (required for driver installation)

### Step 2: Verify Installation

After restarting, verify Npcap is installed:

1. Open PowerShell as Administrator
2. Check if Npcap is running:
   ```powershell
   Get-Service | Where-Object {$_.Name -like "*npcap*"}
   ```

3. Check if the library files exist:
   ```powershell
   Test-Path "C:\Windows\System32\wpcap.dll"
   Test-Path "C:\Windows\System32\Packet.dll"
   ```

### Step 3: Set Environment Variables (if needed)

If the linker still can't find the library, you may need to set environment variables:

1. **Find Npcap installation directory** (usually `C:\Program Files\Npcap`)

2. **Set LIB environment variable**:
   ```powershell
   # In PowerShell (temporary for current session)
   $env:LIB = "C:\Program Files\Npcap\Lib\x64;$env:LIB"
   
   # Or set permanently:
   [System.Environment]::SetEnvironmentVariable("LIB", "C:\Program Files\Npcap\Lib\x64;$env:LIB", "User")
   ```

3. **Set PATH** (usually not needed, but can help):
   ```powershell
   $env:PATH = "C:\Windows\System32;$env:PATH"
   ```

### Step 4: Rebuild

After installing Npcap and restarting:

```bash
cd worker
cargo clean
cargo build --features network-capture
```

## Alternative: Build Without Network Capture

If you don't need network capture right now, you can build without the feature:

```bash
cargo build
cargo run
```

The worker will still function, but HTTP request capture from network packets won't be available.

## Troubleshooting

### Error: "wpcap.dll not found"
- Make sure Npcap is installed
- Restart your computer after installation
- Run the application as Administrator

### Error: Linker can't find wpcap.lib
- Check that Npcap SDK files are installed (usually in `C:\Program Files\Npcap\Lib\x64`)
- Set the `LIB` environment variable as shown above
- Make sure you installed the "Developer's Pack" version if available

### Still having issues?
- Try installing WinPcap (legacy, but sometimes works better): https://www.winpcap.org/
- Or disable network-capture feature for now and use log parsing fallback

