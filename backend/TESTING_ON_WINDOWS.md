# Testing HTTP Request Monitoring on Windows

## Overview

Testing network-level packet capture on Windows has some limitations, but the system **automatically falls back to log parsing**, which works perfectly on Windows!

## Current Status on Windows

✅ **Log Parsing**: Works perfectly (no changes needed)
⏳ **Network Capture**: Limited (Docker Desktop uses WSL2 VM)
✅ **Hybrid Fallback**: Automatically uses log parsing when network capture unavailable

## Testing Options

### Option 1: Test with Log Parsing (Recommended for Windows)

This works immediately on Windows - no setup needed!

#### Step 1: Start the Server
```powershell
cd backend
cargo run
```

#### Step 2: Create a Test Container with HTTP Logging
```powershell
# Run nginx (it logs HTTP requests automatically)
docker run -d -p 8080:80 --name test-nginx nginx

# Or run a simple HTTP server with logging
docker run -d -p 8080:8000 --name test-python \
  python:3.11 python -m http.server 8000
```

#### Step 3: Generate Some Traffic
```powershell
# Make some HTTP requests
curl http://localhost:8080
# Or use PowerShell
Invoke-WebRequest -Uri http://localhost:8080
```

#### Step 4: Check the APM Page
1. Open browser: `http://localhost:8080` (or your frontend URL)
2. Go to APM page
3. Select the container (`test-nginx` or `test-python`)
4. Check "HTTP Requests" section

**Expected Result**: Requests should appear if the container logs HTTP requests in a supported format.

### Option 2: Test Inside WSL2 (For Network Capture)

If you want to test actual network packet capture:

#### Step 1: Install WSL2 and Docker
```powershell
# Enable WSL2 (if not already enabled)
wsl --install

# Install Docker Desktop for Windows
# Download from: https://www.docker.com/products/docker-desktop
```

#### Step 2: Run Server Inside WSL2
```bash
# Open WSL2 terminal
wsl

# Navigate to project
cd /mnt/c/Data/Personal/rust/eyes-devine/backend

# Install dependencies (if needed)
sudo apt-get update
sudo apt-get install libpcap-dev

# Build and run
cargo build
sudo cargo run  # Need sudo for packet capture
```

#### Step 3: Test Network Capture
```bash
# In WSL2, create test container
docker run -d -p 8080:80 nginx

# Generate traffic
curl http://localhost:8080

# Check logs - should see network capture messages
```

### Option 3: Test in Docker Container (Windows)

Run the monitoring server itself in a Docker container with network access:

#### Step 1: Create Dockerfile for Testing
```dockerfile
# Dockerfile.test
FROM rust:1.75

WORKDIR /app
COPY . .

# Install pcap dependencies
RUN apt-get update && apt-get install -y libpcap-dev

# Build
RUN cargo build --release

# Run with network privileges
CMD ["cargo", "run", "--release"]
```

#### Step 2: Run with Network Access
```powershell
# Build
docker build -f Dockerfile.test -t eyes-devine-test .

# Run with network access (requires Docker Desktop)
docker run --rm -it `
  --cap-add=NET_RAW `
  --cap-add=NET_ADMIN `
  --network host `
  -v /var/run/docker.sock:/var/run/docker.sock `
  eyes-devine-test
```

### Option 4: Use Application Logging (Easiest)

Add HTTP logging to your test application:

#### For Node.js Container:
```javascript
// app.js
const express = require('express');
const app = express();

app.use((req, res, next) => {
  const start = Date.now();
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`${req.method} ${req.path} ${res.statusCode} ${duration}ms`);
  });
  next();
});

app.get('/api/test', (req, res) => {
  res.json({ message: 'Hello' });
});

app.listen(3000);
```

#### For Python Container:
```python
# app.py
from flask import Flask
import time

app = Flask(__name__)

@app.before_request
def log_request():
    request.start_time = time.time()

@app.after_request
def log_response(response):
    duration = (time.time() - request.start_time) * 1000
    print(f"{request.method} {request.path} {response.status_code} {duration:.2f}ms")
    return response

@app.route('/api/test')
def test():
    return {'message': 'Hello'}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=3000)
```

## Quick Test Script for Windows

Create `test-http-requests.ps1`:

```powershell
# test-http-requests.ps1
Write-Host "Starting HTTP Request Monitoring Test..."

# Start a test container
Write-Host "Starting nginx container..."
docker run -d -p 8080:80 --name test-nginx nginx

# Wait a moment
Start-Sleep -Seconds 2

# Make some requests
Write-Host "Making HTTP requests..."
1..5 | ForEach-Object {
    Invoke-WebRequest -Uri http://localhost:8080 -UseBasicParsing | Out-Null
    Start-Sleep -Milliseconds 500
}

Write-Host "Test complete! Check the APM page for HTTP requests."
Write-Host "Container name: test-nginx"
Write-Host "Clean up with: docker rm -f test-nginx"
```

Run it:
```powershell
.\test-http-requests.ps1
```

## Verifying It Works

### Check Server Logs
Look for these messages:
```
Starting network-level HTTP request monitoring
No Docker network interfaces found. Network monitoring may not work.
Falling back to log parsing
```

### Check APM Page
1. Open frontend: `http://localhost:3000` (or your frontend port)
2. Navigate to APM page
3. Select your test container
4. Look at "HTTP Requests" section

### Expected Behavior

**If log parsing works:**
- ✅ Requests appear in the table
- ✅ Shows method, endpoint, status, response time
- ✅ Updates in real-time

**If no requests appear:**
- Check container logs: `docker logs test-nginx`
- Verify log format matches supported patterns
- See `SETUP_HTTP_MONITORING.md` for log format examples

## Troubleshooting

### Problem: "No HTTP requests found"
**Solution**: 
- Check if container logs HTTP requests: `docker logs <container>`
- Add HTTP logging middleware to your application
- See `SETUP_HTTP_MONITORING.md` for examples

### Problem: Network capture not working
**Solution**: 
- This is expected on Windows! System automatically uses log parsing
- Log parsing works perfectly on Windows
- For network capture, use WSL2 or Docker container

### Problem: Can't see Docker containers
**Solution**:
- Ensure Docker Desktop is running
- Check Docker connection: `docker ps`
- Verify Docker socket is accessible

## Best Practice for Windows Development

1. **Use Log Parsing**: It works perfectly on Windows, no setup needed
2. **Add HTTP Logging**: Add simple logging middleware to your apps
3. **Test Locally**: Everything works without network capture
4. **Deploy with Network Capture**: Use Linux/WSL2 for production-like testing

## Summary

✅ **Log Parsing**: Works on Windows immediately
✅ **Automatic Fallback**: System handles Windows gracefully  
✅ **No Setup Needed**: Just run `cargo run` and test
⏳ **Network Capture**: Requires WSL2 or Docker container (optional)

The system is designed to work on Windows out of the box using log parsing!

