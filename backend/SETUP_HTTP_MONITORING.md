# Setting Up HTTP Request Monitoring

## Current Limitation

The current system parses HTTP requests from container logs. If your application doesn't log HTTP requests, or uses a different log format, requests won't be captured.

## Quick Solutions

### Solution 1: Add HTTP Logging to Your Application

#### For Node.js/Express:
```javascript
app.use((req, res, next) => {
  const start = Date.now();
  res.on('finish', () => {
    const duration = Date.now() - start;
    console.log(`${req.method} ${req.path} ${res.statusCode} ${duration}ms`);
  });
  next();
});
```

#### For Python/Flask:
```python
import time
from flask import Flask, request

@app.before_request
def log_request():
    request.start_time = time.time()

@app.after_request
def log_response(response):
    duration = (time.time() - request.start_time) * 1000
    print(f"{request.method} {request.path} {response.status_code} {duration:.2f}ms")
    return response
```

#### For Rust/Actix-web:
```rust
use actix_web::middleware::Logger;
use env_logger;

// In main():
env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
app.wrap(Logger::default()); // Logs: "GET /api/users 200 45ms"
```

#### For Go/Gin:
```go
func Logger() gin.HandlerFunc {
    return func(c *gin.Context) {
        start := time.Now()
        c.Next()
        duration := time.Since(start)
        log.Printf("%s %s %d %v", c.Request.Method, c.Request.URL.Path, 
                   c.Writer.Status(), duration)
    }
}
```

### Solution 2: Use Structured JSON Logging

Many frameworks support JSON logging which is easier to parse:

#### Example JSON Log Format:
```json
{"method":"GET","path":"/api/users","status":200,"duration":45.2,"timestamp":"2024-12-25T10:00:00Z"}
```

The system already supports parsing this format!

### Solution 3: Deploy a Reverse Proxy (Recommended for Production)

#### Using Nginx

1. **Create nginx configuration:**
```nginx
# nginx.conf
http {
    log_format json_combined escape=json
        '{'
            '"time":"$time_iso8601",'
            '"method":"$request_method",'
            '"path":"$request_uri",'
            '"status":$status,'
            '"duration":$request_time,'
            '"remote_addr":"$remote_addr"'
        '}';
    
    access_log /var/log/nginx/access.log json_combined;
    
    upstream app {
        server app:8080;
    }
    
    server {
        listen 80;
        location / {
            proxy_pass http://app;
        }
    }
}
```

2. **Update docker-compose.yml:**
```yaml
services:
  nginx:
    image: nginx
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - nginx-logs:/var/log/nginx
    ports:
      - "80:80"
    depends_on:
      - app
  
  app:
    # Your application
    # Remove port mapping (nginx handles it)
```

3. **Mount nginx logs to monitoring container:**
```yaml
  monitor:
    volumes:
      - nginx-logs:/var/log/nginx:ro
```

#### Using Traefik

Traefik automatically logs all requests in structured format:

```yaml
services:
  traefik:
    image: traefik:v2.10
    command:
      - "--api.insecure=true"
      - "--providers.docker=true"
      - "--accesslog=true"
      - "--accesslog.format=json"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    ports:
      - "80:80"
      - "8080:8080" # Traefik dashboard
```

### Solution 4: Network-Level Monitoring (Future)

For a solution that works without any application changes, network-level packet capture can be implemented. This requires:

- Root/capabilities to capture packets
- Parsing HTTP from network packets
- Handling encrypted traffic (HTTPS)

See `HTTP_REQUEST_MONITORING.md` for details on this approach.

## Testing Your Setup

1. **Check if requests are being logged:**
```bash
docker logs <container-id> | grep -E "(GET|POST|PUT|DELETE)"
```

2. **Verify log format matches supported patterns:**
- Simple: `GET /api/users 200 45ms`
- Apache: `"GET /api/health HTTP/1.1" 200`
- JSON: `{"method":"GET","path":"/api/users","status":200,"duration":45.2}`

3. **Check the APM page:**
- Select your service
- Look at the "HTTP Requests" section
- If empty, check the helpful message for guidance

## Troubleshooting

**Problem:** No requests showing up
- **Check:** Are requests being logged? `docker logs <container>`
- **Check:** Does log format match supported patterns?
- **Solution:** Add logging middleware or use reverse proxy

**Problem:** Requests show but missing fields
- **Check:** Log format includes method, path, status, duration
- **Solution:** Update logging to include all required fields

**Problem:** Different framework not supported
- **Solution:** Add custom regex pattern to `parse_http_request_from_log()` function
- Or: Use structured JSON logging (most reliable)

## Best Practices

1. **Use structured logging** (JSON format) - easiest to parse
2. **Include all fields:** method, path, status, duration
3. **Use reverse proxy** for production - centralized logging
4. **Log to stdout/stderr** - Docker captures these automatically
5. **Avoid logging sensitive data** in request paths (query params, headers)

