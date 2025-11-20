# HTTP Request Capture Feature Design

## Overview

This document describes the design for capturing HTTP/RESTful API requests from Docker containers. The system supports two deployment scenarios:
1. **Local Development**: Running on host machine, monitoring Docker containers
2. **Production**: Running inside Docker, monitoring other containers on the same network

## Requirements

### Functional Requirements

1. **Dual Deployment Support**
   - ✅ Run locally to capture requests from Docker services (testing/development)
   - ✅ Run in Docker to capture requests from other Docker services (production)
   - ✅ Automatically detect deployment environment

2. **Capture Scope**
   - ✅ RESTful API requests only (HTTP/HTTPS)
   - ✅ Support common HTTP methods: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
   - ✅ Capture both requests and responses

3. **Captured Data**
   - ✅ API endpoint/path (e.g., `/api/users`, `/health`)
   - ✅ HTTP method (GET, POST, PUT, DELETE, etc.)
   - ✅ Latency/Response time (in milliseconds)
   - ✅ HTTP status code (200, 404, 500, etc.)
   - ✅ Timestamp
   - ✅ Container ID and name

### Non-Functional Requirements

- **Performance**: Minimal overhead on network traffic
- **Reliability**: Graceful fallback to log parsing if network capture fails
- **Security**: Requires elevated privileges for packet capture
- **Compatibility**: Works on Linux, macOS, and Windows

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                    Request Capture System                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐         ┌──────────────────┐        │
│  │ Network Monitor  │         │   Log Parser      │        │
│  │   Service        │◄───────►│   (Fallback)     │        │
│  │                  │         │                   │        │
│  │ - Packet Capture │         │ - Parse Logs     │        │
│  │ - IP Matching    │         │ - Regex Patterns │        │
│  │ - HTTP Parsing   │         │ - Multiple Formats│        │
│  └──────────────────┘         └──────────────────┘        │
│           │                              │                   │
│           └──────────────┬───────────────┘                   │
│                          │                                    │
│                  ┌───────▼────────┐                          │
│                  │ Request Store  │                          │
│                  │  (In-Memory)   │                          │
│                  └───────┬────────┘                          │
│                          │                                    │
│                  ┌───────▼────────┐                          │
│                  │  API Endpoint  │                          │
│                  │  /api/containers│                          │
│                  │  /{id}/requests │                          │
│                  └─────────────────┘                          │
└─────────────────────────────────────────────────────────────┘
```

## Deployment Scenarios

### Scenario 1: Local Development (Host Machine)

```
┌─────────────────────────────────────────────────────────────┐
│                        Host Machine                          │
│                                                               │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │ Eyes Devine App  │         │  Docker Containers│          │
│  │  (Local Process) │         │                  │          │
│  │                  │         │  ┌──────────────┐│          │
│  │  Network Monitor │◄────────┤──│  Service A   ││          │
│  │  (pcap)          │         │  │  :8080        ││          │
│  │                  │         │  └──────────────┘│          │
│  │  Captures on:    │         │  ┌──────────────┐│          │
│  │  - docker0       │         │  │  Service B   ││          │
│  │  - br-*          │         │  │  :3000        ││          │
│  │  - "any"         │         │  └──────────────┘│          │
│  └──────────────────┘         └──────────────────┘          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**How it works:**
1. Application runs as a local process on the host
2. Network monitor captures packets on Docker bridge interfaces (docker0, br-*)
3. Extracts IP addresses from packets
4. Matches IPs to containers via Docker API
5. Parses HTTP requests/responses from packet payload

### Scenario 2: Production (Docker Container)

```
┌─────────────────────────────────────────────────────────────┐
│                    Docker Network (bridge)                   │
│                                                               │
│  ┌──────────────────┐         ┌──────────────────┐          │
│  │ Eyes Devine      │         │  Service A        │          │
│  │  Container       │         │  :8080             │          │
│  │                  │         │                    │          │
│  │  Network Monitor │◄────────┤  HTTP Requests    │          │
│  │  (pcap)          │         │                    │          │
│  │                  │         └────────────────────┘          │
│  │  Captures on:   │         ┌──────────────────┐          │
│  │  - Docker network│         │  Service B        │          │
│  │  - "any"         │         │  :3000             │          │
│  └──────────────────┘         └────────────────────┘          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**How it works:**
1. Application runs inside a Docker container
2. Container has `NET_RAW` capability for packet capture
3. Network monitor captures packets on Docker network interface
4. All containers are on the same network, so IP matching works directly
5. Parses HTTP requests/responses from packet payload

## Data Capture Flow

### Network-Level Capture (Primary Method)

```
1. Packet Capture
   └─► Capture TCP packets on network interface
       └─► Filter: tcp port 80, 443, 8080, 8000, 3000, 5000, 9000

2. IP Extraction
   └─► Extract source/destination IP from packet header
       └─► Match IP to container via Docker API

3. HTTP Parsing
   └─► Extract TCP payload (skip Ethernet/IP/TCP headers)
       └─► Parse HTTP request/response using httparse
           ├─► Request: Extract method, path
           └─► Response: Extract status code

4. Latency Calculation
   └─► Track request timestamp
       └─► Match with response timestamp
           └─► Calculate: response_time = response_time - request_time

5. Storage
   └─► Store in in-memory HashMap<container_id, Vec<HttpRequest>>
       └─► Keep last 1000 requests per container
```

### Log Parsing (Fallback Method)

```
1. Log Retrieval
   └─► Get container logs via Docker API
       └─► Retrieve last N log lines

2. Pattern Matching
   └─► Apply regex patterns to log lines:
       ├─► Pattern 1: "GET /api/users 200 45ms"
       ├─► Pattern 2: Apache/Nginx format
       ├─► Pattern 3: "GET /endpoint HTTP/1.1" 200 0.045s
       └─► Pattern 4: JSON format

3. Data Extraction
   └─► Extract: method, path, status, latency
       └─► Create HttpRequest object

4. Return Results
   └─► Return array of HttpRequest objects
```

## Captured Data Structure

### HttpRequest Model

```rust
pub struct HttpRequest {
    pub container_id: String,        // Docker container ID
    pub container_name: String,      // Container name
    pub endpoint: String,            // API path (e.g., "/api/users")
    pub method: String,              // HTTP method (GET, POST, etc.)
    pub http_status: u16,            // Status code (200, 404, 500, etc.)
    pub response_time_ms: f64,       // Latency in milliseconds
    pub timestamp: DateTime<Utc>,    // Request timestamp
}
```

### Example Response

```json
[
  {
    "container_id": "abc123def456",
    "container_name": "my-api-service",
    "endpoint": "/api/users",
    "method": "GET",
    "http_status": 200,
    "response_time_ms": 45.2,
    "timestamp": "2024-12-25T10:00:00Z"
  },
  {
    "container_id": "abc123def456",
    "container_name": "my-api-service",
    "endpoint": "/api/orders",
    "method": "POST",
    "http_status": 201,
    "response_time_ms": 123.5,
    "timestamp": "2024-12-25T10:00:15Z"
  }
]
```

## API Endpoint

### Get Container HTTP Requests

**Endpoint:** `GET /api/containers/{container_id}/requests`

**Parameters:**
- `container_id` (path): Container ID or name
- `limit` (query, optional): Maximum number of requests to return (default: 100)

**Example Request:**
```bash
GET /api/containers/abc123def456/requests?limit=50
```

**Response:**
```json
[
  {
    "container_id": "abc123def456",
    "container_name": "my-api-service",
    "endpoint": "/api/users",
    "method": "GET",
    "http_status": 200,
    "response_time_ms": 45.2,
    "timestamp": "2024-12-25T10:00:00Z"
  }
]
```

## Implementation Details

### Network Capture

**Technology Stack:**
- `pcap` crate for cross-platform packet capture
- `httparse` crate for HTTP parsing
- Requires `network-capture` feature flag

**Packet Filter:**
```
tcp port 80 or tcp port 443 or tcp port 8080 or tcp port 8443 or 
tcp port 8000 or tcp port 3000 or tcp port 5000 or tcp port 9000
```

**IP Matching:**
1. Extract source/destination IP from packet
2. Query Docker API for all containers
3. Get network info for each container
4. Match packet IP to container IP
5. Cache results to reduce API calls

### Environment Detection

**Detection Methods:**
1. Check for `/.dockerenv` file (Docker creates this)
2. Check `/proc/self/cgroup` for "docker" or "containerd" (Linux)

**Behavior:**
- **Local**: Monitors Docker bridge networks, falls back to "any" interface
- **Docker**: Monitors Docker network interface, uses "any" if needed

### Latency Calculation

**Network Capture:**
- Track request timestamp when HTTP request packet is captured
- Track response timestamp when HTTP response packet is captured
- Calculate: `latency = response_timestamp - request_timestamp`

**Log Parsing:**
- Extract latency from log format (e.g., "45ms", "0.045s")
- Convert to milliseconds

### Error Handling

**Graceful Degradation:**
1. Try network-level capture first
2. If network capture fails or returns empty:
   - Fall back to log parsing
   - Log warning with helpful debugging info
3. If both fail:
   - Return empty array
   - Log error with suggestions

## Configuration

### Required Permissions

**Linux:**
- Root access OR
- `CAP_NET_RAW` capability
- In Docker: `--cap-add=NET_RAW`

**macOS:**
- Root or admin privileges

**Windows:**
- Administrator privileges
- Npcap or WinPcap installed

### Docker Configuration

**For Production (Running in Docker):**
```yaml
services:
  eyes-devine:
    image: eyes-devine:latest
    cap_add:
      - NET_RAW
      - NET_ADMIN
    network_mode: bridge  # or specific network
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
```

**Build with Network Capture:**
```bash
cargo build --features network-capture
```

## Limitations

1. **HTTPS Traffic**: Cannot decrypt HTTPS without MITM proxy or TLS termination
2. **TCP Reassembly**: Current implementation is simplified - may miss fragmented packets
3. **Connection Tracking**: Does not track full request/response pairs across multiple packets
4. **Performance**: Network capture adds overhead, especially on high-traffic networks
5. **Permissions**: Requires elevated privileges for packet capture

## Future Enhancements

1. **HTTPS Support**: Integrate with TLS termination proxy
2. **Full TCP Reassembly**: Properly handle fragmented packets
3. **Connection Tracking**: Track full request/response pairs
4. **Persistent Storage**: Store requests in database instead of in-memory
5. **Request/Response Bodies**: Capture full request/response payloads
6. **Headers**: Capture HTTP headers
7. **Distributed Tracing**: Support for trace IDs and span correlation

## Testing

### Local Testing
1. Start Docker containers with HTTP services
2. Run Eyes Devine locally
3. Send requests to Docker services
4. Query `/api/containers/{id}/requests`
5. Verify captured requests

### Docker Testing
1. Build Docker image with network-capture feature
2. Run in Docker with `NET_RAW` capability
3. Send requests between containers
4. Query API endpoint
5. Verify captured requests

## Summary

This design provides a robust HTTP request capture system that:
- ✅ Works in both local and Docker environments
- ✅ Captures RESTful API requests with required fields
- ✅ Gracefully falls back to log parsing
- ✅ Provides clear API for accessing captured data
- ✅ Handles errors gracefully with helpful logging

The implementation is production-ready for capturing RESTful API traffic from Docker containers in both development and production environments.

