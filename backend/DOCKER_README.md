# Docker Setup for Eyes Devine Backend

## Quick Start

### Build the Docker Image

```bash
cd backend
docker build -t eyes-devine-server .
```

### Run the Container

```bash
# Basic run (without database/redis)
docker run -d \
  --name eyes-devine-server \
  -p 8080:8080 \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  eyes-devine-server
```

### Run with Docker Compose (Recommended)

```bash
# Copy the example compose file
cp docker-compose.example.yml docker-compose.yml

# Edit docker-compose.yml with your settings
# Then run:
docker-compose up -d
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` | Server bind address |
| `SERVER_PORT` | `8080` | Server port |
| `DATABASE_URL` | (none) | PostgreSQL connection string |
| `REDIS_URL` | (none) | Redis connection string (optional) |
| `DOCKER_HOST` | `unix:///var/run/docker.sock` | Docker daemon connection |
| `CACHE_TTL_CONTAINERS` | `10` | Container cache TTL (seconds) |
| `CACHE_TTL_STATS` | `2` | Stats cache TTL (seconds) |
| `CACHE_TTL_IMAGES` | `300` | Images cache TTL (seconds) |
| `CACHE_TTL_HISTORY` | `30` | History cache TTL (seconds) |
| `MAX_QUERY_RANGE_DAYS` | `30` | Maximum query range in days |
| `MAX_RESULTS_PER_QUERY` | `10000` | Maximum results per query |
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |

### Docker Socket Access

The container needs access to the Docker socket to monitor containers. Mount it as:

```bash
-v /var/run/docker.sock:/var/run/docker.sock:ro
```

**Note**: On Windows with Docker Desktop, the socket path is different. Use:
```bash
-v //var/run/docker.sock:/var/run/docker.sock:ro
```

### Network Packet Capture (Linux Only)

For network-level HTTP request monitoring, the container needs additional capabilities:

```yaml
cap_add:
  - NET_RAW
  - NET_ADMIN
```

Or use host network mode:
```yaml
network_mode: host
```

**Note**: Network packet capture only works on Linux. On Windows/macOS, the system automatically falls back to log parsing.

## Building

### Build Arguments

- `RUST_VERSION`: Rust version to use (default: `1.75.0`)
- `APP_NAME`: Binary name (default: `eyes-devine-server`)

Example:
```bash
docker build \
  --build-arg RUST_VERSION=1.75.0 \
  --build-arg APP_NAME=eyes-devine-server \
  -t eyes-devine-server .
```

### Multi-stage Build

The Dockerfile uses a multi-stage build:
1. **Build stage**: Compiles the Rust application with all dependencies
2. **Runtime stage**: Minimal Debian image with only runtime dependencies

This results in a smaller final image (~100MB vs ~1GB+).

## Health Check

The container includes a health check that verifies the API is responding:

```bash
# Check health status
docker ps  # Look for "healthy" status

# Or manually check
curl http://localhost:8080/api/stats/total
```

## Troubleshooting

### Container won't start

1. **Check logs**:
   ```bash
   docker logs eyes-devine-server
   ```

2. **Verify Docker socket access**:
   ```bash
   docker exec eyes-devine-server ls -la /var/run/docker.sock
   ```

3. **Check environment variables**:
   ```bash
   docker exec eyes-devine-server env
   ```

### Can't connect to Docker daemon

- Ensure Docker socket is mounted correctly
- On Windows, use `//var/run/docker.sock` path
- Check socket permissions (should be readable)

### Network capture not working

- Network capture requires Linux
- On Windows/macOS, system uses log parsing (works fine)
- If on Linux, ensure `NET_RAW` and `NET_ADMIN` capabilities are added

### Database connection issues

- Verify `DATABASE_URL` is correct
- Ensure database container is running and accessible
- Check network connectivity between containers

## Development

### Build for Development

```bash
# Build without optimization (faster)
docker build \
  --target build \
  -t eyes-devine-server:dev .
```

### Run with Volume Mount (for development)

```bash
docker run -it --rm \
  -p 8080:8080 \
  -v $(pwd):/app \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  eyes-devine-server:dev \
  cargo run
```

## Production Deployment

### Security Considerations

1. **Run as non-root user**: The image already runs as `appuser` (UID 1000)
2. **Read-only Docker socket**: Use `:ro` flag when mounting
3. **Network isolation**: Use Docker networks to isolate services
4. **Secrets management**: Use Docker secrets or environment variable files
5. **Resource limits**: Set CPU and memory limits

### Example Production Setup

```yaml
services:
  eyes-devine-server:
    image: eyes-devine-server:latest
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
    security_opt:
      - no-new-privileges:true
    read_only: true
    tmpfs:
      - /tmp
```

## Image Size

- **Build stage**: ~1.5GB (includes Rust toolchain)
- **Runtime stage**: ~150MB (minimal Debian + dependencies)
- **Final image**: ~150MB

## Dependencies

The runtime image includes:
- `libssl3` - SSL/TLS support
- `libpq5` - PostgreSQL client library
- `libpcap0.8` - Network packet capture (Linux)
- `ca-certificates` - SSL certificate validation
- `curl` - Health check utility

