# Cross-Platform Network Packet Capture

## Overview

The network monitoring service now supports **cross-platform packet capture** on Linux, macOS, and Windows. It uses the `pcap` crate which provides bindings to:
- **Linux**: `libpcap`
- **macOS**: `libpcap` (built-in)
- **Windows**: `Npcap` or `WinPcap`

## Platform-Specific Setup

### Linux

#### Installation
```bash
# Ubuntu/Debian
sudo apt-get install libpcap-dev

# CentOS/RHEL
sudo yum install libpcap-devel

# Alpine (for Docker containers)
apk add libpcap-dev
```

#### Running with Privileges

**Option 1: Run as Root (Development)**
```bash
sudo cargo run
```

**Option 2: Use Capabilities (Production)**
```bash
# Grant CAP_NET_RAW capability
sudo setcap cap_net_raw,cap_net_admin=eip target/debug/eyes-devine-server
cargo run
```

**Option 3: Docker Container with Privileges**
```yaml
services:
  monitor:
    image: eyes-devine-server
    cap_add:
      - NET_RAW
      - NET_ADMIN
    network_mode: host  # Required to access network interfaces
```

### macOS

#### Installation
`libpcap` is built into macOS, but you may need Xcode Command Line Tools:
```bash
xcode-select --install
```

#### Running with Privileges
```bash
# Run as root (requires password)
sudo cargo run

# Or grant admin privileges to the binary
sudo chown root:wheel target/debug/eyes-devine-server
sudo chmod +s target/debug/eyes-devine-server
```

#### Docker Desktop Considerations
On macOS, Docker Desktop runs in a VM. Network capture will work on:
- The host macOS network interfaces (en0, en1, etc.)
- Docker bridge interfaces if running inside a container with proper privileges

### Windows

#### Installation

**Step 1: Install Npcap (Recommended)**
1. Download Npcap from: https://nmap.org/npcap/
2. Install with "WinPcap API-compatible Mode" enabled
3. Restart your computer

**Alternative: WinPcap (Legacy)**
- Download from: https://www.winpcap.org/
- Note: WinPcap is no longer maintained, Npcap is recommended

#### Running with Privileges
```powershell
# Run PowerShell as Administrator
# Then run:
cargo run
```

#### Docker Desktop Considerations
On Windows, Docker Desktop uses WSL2 or Hyper-V. Network capture will work on:
- Windows network adapters (Ethernet, Wi-Fi, etc.)
- Docker Desktop virtual adapters (vEthernet (WSL), DockerNAT)
- If running inside WSL2, use Linux instructions

## Interface Detection

The service automatically detects Docker network interfaces:

**Linux:**
- `docker0` (default bridge)
- `br-*` (custom bridges)
- `veth*` (virtual ethernet pairs)

**macOS:**
- `bridge0` (Docker bridge)
- `en0`, `en1` (physical interfaces)
- `vmnet*` (VMware/virtual interfaces)

**Windows:**
- `vEthernet (WSL)` (Docker Desktop WSL2)
- `DockerNAT` (Docker Desktop Hyper-V)
- `Ethernet`, `Wi-Fi` (physical adapters)

If no Docker-specific interfaces are found, the service will:
1. List all available interfaces
2. Attempt to capture on common default interfaces
3. Log instructions for manual interface selection

## Manual Interface Selection

If automatic detection fails, you can manually specify interfaces by:

1. **Check available interfaces:**
   ```bash
   # Linux/macOS
   ip link show
   # or
   ifconfig
   
   # Windows
   Get-NetAdapter
   ```

2. **The service will log all available interfaces at startup**

3. **For production, consider adding environment variable support:**
   ```bash
   NETWORK_INTERFACES=docker0,br-12345
   ```

## Testing

### Test on Linux
```bash
# Start a test container
docker run -d -p 8080:80 nginx

# Make some requests
curl http://localhost:8080

# Check logs for captured requests
```

### Test on macOS
```bash
# Start a test container
docker run -d -p 8080:80 nginx

# Make some requests
curl http://localhost:8080

# The service should capture on en0 or bridge0
```

### Test on Windows
```powershell
# Start a test container
docker run -d -p 8080:80 nginx

# Make some requests
Invoke-WebRequest http://localhost:8080

# The service should capture on Ethernet or DockerNAT adapter
```

## Troubleshooting

### "Permission denied" Error

**Linux:**
- Run with `sudo` or grant `CAP_NET_RAW` capability
- Check: `getcap target/debug/eyes-devine-server`

**macOS:**
- Run with `sudo`
- Grant admin privileges to the binary

**Windows:**
- Run PowerShell/Command Prompt as Administrator
- Ensure Npcap/WinPcap is installed and running
- Check Windows Firewall settings

### "Interface not found" Error

1. List available interfaces (see above)
2. Check Docker network interfaces:
   ```bash
   docker network ls
   docker network inspect <network_name>
   ```
3. The service logs all available interfaces at startup

### No Packets Captured

1. **Check filter**: The service filters for ports 80, 8080, 8000, 3000, 5000
2. **Verify traffic**: Use `tcpdump` or Wireshark to verify traffic exists
3. **Check privileges**: Ensure running with proper permissions
4. **Docker Desktop**: On macOS/Windows, Docker runs in a VM - capture on host interfaces may not see container traffic

### Docker Desktop VM Limitation

On macOS and Windows, Docker Desktop runs containers in a VM. To capture container traffic:

**Option 1: Run the monitor inside Docker**
```yaml
services:
  monitor:
    build: .
    network_mode: host  # Linux
    # or
    cap_add:
      - NET_RAW
      - NET_ADMIN
```

**Option 2: Use WSL2 (Windows)**
- Run the server inside WSL2
- Capture on WSL2 network interfaces

**Option 3: Monitor host interfaces**
- Capture on the host's network adapters
- This will see traffic to/from the host, but may miss inter-container traffic

## Performance Considerations

- **Packet capture is CPU-intensive**: Consider limiting capture to specific interfaces
- **Memory usage**: The service keeps the last 1000 requests per container in memory
- **Network overhead**: Promiscuous mode may impact network performance
- **Filter early**: BPF filters are applied at kernel level for efficiency

## Security Notes

- Packet capture requires elevated privileges
- Promiscuous mode can capture all network traffic (security concern)
- Consider running in a dedicated monitoring container with limited privileges
- For production, use capabilities (Linux) or service accounts (Windows) instead of full root/admin

## Next Steps

1. **TCP Stream Reassembly**: Currently handles single packets; HTTP can span multiple packets
2. **Container IP Mapping**: Map captured packets to containers by IP address
3. **HTTPS Support**: Decrypt TLS traffic (requires certificates or MITM proxy)
4. **Persistent Storage**: Store requests in database instead of in-memory only

