# Docker Monitoring System - Architecture Design

## Overview
A high-performance, non-blocking Docker monitoring system similar to DataDog, with separate worker and API services for collecting and serving Docker metrics, container status, and image information.

## System Architecture

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│   Docker API    │◄────────┤   Worker        │────────►│  TimescaleDB    │
│   (bollard)     │         │   Service       │         │  (PostgreSQL)   │
└─────────────────┘         └─────────────────┘         └─────────────────┘
                                      │                           ▲
                                      │                           │
                                      │                           │
                            ┌─────────▼──────────┐               │
                            │   Redis Cache      │               │
                            │   (Optional)       │               │
                            └────────────────────┘               │
                                      ▲                           │
                                      │                           │
┌─────────────────┐         ┌────────┴──────────┐                  │
│   Frontend      │◄────────┤   API Service   │──────────────────┘
│   (Leptos)      │         │   (actix-web)   │
└─────────────────┘         └─────────────────┘
```

## Core Principles

1. **Separation of Concerns**: Worker collects, API serves
2. **Non-Blocking**: Worker never blocks API requests
3. **High Throughput**: Batch inserts, connection pooling
4. **Low Latency**: Caching, optimized queries, read replicas (future)
5. **Time-Series Optimized**: TimescaleDB for efficient time-range queries

## Components

### 1. Worker Service

**Responsibilities:**
- Collect Docker metrics (stats, status, images) periodically
- Store data in TimescaleDB with batch inserts
- Handle errors gracefully without crashing
- Support configurable collection intervals

**Collection Strategy:**
- **Container Stats**: Every 5-10 seconds (configurable)
- **Container Status**: Every 30 seconds (less frequent)
- **Image Information**: On startup + when new images detected
- **Container Logs**: Stream and store in real-time (optional)

**Implementation Details:**
- Use separate async tasks for different collection types
- Batch inserts (100-1000 records per batch) for performance
- Use connection pooling (sqlx pool)
- Implement backpressure handling
- Retry logic with exponential backoff

### 2. API Service

**Responsibilities:**
- Serve data from database (NOT directly from Docker)
- Provide REST endpoints for:
  - Container lists and details
  - Historical metrics (time-series queries)
  - Image information
  - Logs (from database)
- Implement caching layer (Redis) for frequently accessed data
- Support pagination and filtering

**Performance Optimizations:**
- Redis cache for:
  - Current container list (TTL: 5-10 seconds)
  - Recent stats (TTL: 1-2 seconds)
  - Image metadata (TTL: 5 minutes)
- Database query optimization:
  - Use TimescaleDB continuous aggregates for pre-computed metrics
  - Indexes on (container_id, timestamp) for time-range queries
  - Limit query result sizes

### 3. Database Schema (TimescaleDB)

**Time-Series Tables:**
- `container_stats` - Convert to hypertable
- `container_logs` - Convert to hypertable
- `container_info` - Regular table (snapshot data)

**New Tables:**
- `docker_images` - Image metadata and tracking
- `image_versions` - Track image changes over time

**Optimizations:**
- Convert time-series tables to TimescaleDB hypertables
- Create continuous aggregates for:
  - Hourly/daily aggregated stats
  - Container uptime calculations
- Retention policies (e.g., keep raw data for 7 days, aggregated for 90 days)
- Compression policies for old data

### 4. Caching Layer (Redis - Optional but Recommended)

**Cache Strategy:**
- **Container List**: Cache for 5-10 seconds
- **Recent Stats**: Cache for 1-2 seconds
- **Image Metadata**: Cache for 5 minutes
- **Historical Queries**: Cache for 30 seconds (if query is expensive)

**Cache Keys:**
- `containers:list` - Current container list
- `stats:container:{id}:latest` - Latest stats for container
- `stats:total:latest` - Latest total stats
- `image:{id}` - Image metadata
- `query:stats:{container_id}:{from}:{to}` - Historical stats query

## Data Flow

### Collection Flow (Worker)
```
1. Worker polls Docker API
2. Collects metrics/stats/images
3. Batches data (100-1000 records)
4. Inserts into TimescaleDB (batch insert)
5. Updates Redis cache (optional)
6. Sleep for interval
```

### Query Flow (API)
```
1. API receives request
2. Check Redis cache
3. If cache miss:
   a. Query TimescaleDB
   b. Store in cache
   c. Return data
4. If cache hit:
   a. Return cached data
```

## Database Schema Design

### TimescaleDB Hypertables

#### `container_stats` (Hypertable)
```sql
CREATE TABLE container_stats (
    id BIGSERIAL,
    container_id VARCHAR(255) NOT NULL,
    container_name VARCHAR(255) NOT NULL,
    cpu_usage_percent DOUBLE PRECISION NOT NULL,
    memory_usage_bytes BIGINT NOT NULL,
    memory_limit_bytes BIGINT NOT NULL,
    memory_usage_percent DOUBLE PRECISION NOT NULL,
    network_rx_bytes BIGINT NOT NULL,
    network_tx_bytes BIGINT NOT NULL,
    block_read_bytes BIGINT NOT NULL,
    block_write_bytes BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable
SELECT create_hypertable('container_stats', 'timestamp');

-- Indexes
CREATE INDEX idx_container_stats_container_timestamp 
    ON container_stats (container_id, timestamp DESC);
```

#### `container_logs` (Hypertable)
```sql
-- Similar structure, convert to hypertable
SELECT create_hypertable('container_logs', 'timestamp');
```

#### `docker_images` (New Table)
```sql
CREATE TABLE docker_images (
    id BIGSERIAL PRIMARY KEY,
    image_id VARCHAR(255) NOT NULL UNIQUE,
    repo_tags TEXT[] NOT NULL,
    size_bytes BIGINT NOT NULL,
    architecture VARCHAR(50),
    os VARCHAR(50),
    created_at TIMESTAMPTZ,
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_docker_images_image_id ON docker_images(image_id);
CREATE INDEX idx_docker_images_repo_tags ON docker_images USING GIN(repo_tags);
```

#### `image_versions` (Time-Series - Track Image Changes)
```sql
CREATE TABLE image_versions (
    id BIGSERIAL,
    image_id VARCHAR(255) NOT NULL,
    repo_tags TEXT[] NOT NULL,
    size_bytes BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

SELECT create_hypertable('image_versions', 'timestamp');
```

### Continuous Aggregates (Pre-computed Metrics)

```sql
-- Hourly aggregated stats
CREATE MATERIALIZED VIEW container_stats_hourly
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', timestamp) AS bucket,
    container_id,
    container_name,
    AVG(cpu_usage_percent) AS avg_cpu,
    MAX(cpu_usage_percent) AS max_cpu,
    AVG(memory_usage_bytes) AS avg_memory,
    MAX(memory_usage_bytes) AS max_memory,
    SUM(network_rx_bytes) AS total_network_rx,
    SUM(network_tx_bytes) AS total_network_tx
FROM container_stats
GROUP BY bucket, container_id, container_name;

-- Add refresh policy (refresh every hour)
SELECT add_continuous_aggregate_policy('container_stats_hourly',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour');
```

### Retention Policies

```sql
-- Keep raw stats for 7 days
SELECT add_retention_policy('container_stats', INTERVAL '7 days');

-- Keep raw logs for 3 days
SELECT add_retention_policy('container_logs', INTERVAL '3 days');

-- Keep image versions for 30 days
SELECT add_retention_policy('image_versions', INTERVAL '30 days');
```

## API Endpoints Design

### Container Endpoints
- `GET /api/containers` - List all containers (from DB, cached)
- `GET /api/containers/{id}` - Get container details
- `GET /api/containers/{id}/stats` - Get latest stats
- `GET /api/containers/{id}/stats/history` - Get historical stats (time range)
- `GET /api/containers/{id}/logs` - Get logs (from DB)

### Image Endpoints
- `GET /api/images` - List all images
- `GET /api/images/{id}` - Get image details
- `GET /api/images/{id}/history` - Get image version history

### Metrics Endpoints
- `GET /api/metrics/total` - Get total aggregated stats
- `GET /api/metrics/total/history` - Get historical total stats
- `GET /api/metrics/containers` - Get stats for all containers

### SSE Endpoints (Real-time)
- `GET /api/stream/stats` - Stream latest stats (from cache/DB)
- `GET /api/stream/containers` - Stream container updates

## Configuration

### Worker Configuration
```rust
pub struct WorkerConfig {
    // Collection intervals
    pub stats_collection_interval: Duration,      // Default: 5s
    pub status_collection_interval: Duration,      // Default: 30s
    pub image_collection_interval: Duration,       // Default: 60s
    
    // Batch settings
    pub batch_size: usize,                        // Default: 500
    pub batch_timeout: Duration,                  // Default: 1s
    
    // Database
    pub database_url: String,
    
    // Docker
    pub docker_socket_path: Option<String>,
}
```

### API Configuration
```rust
pub struct ApiConfig {
    pub database_url: String,
    pub redis_url: Option<String>,  // Optional
    
    // Cache TTLs
    pub cache_ttl_containers: Duration,    // Default: 10s
    pub cache_ttl_stats: Duration,         // Default: 2s
    pub cache_ttl_images: Duration,        // Default: 5min
    
    // Query limits
    pub max_query_range_days: u32,        // Default: 30
    pub max_results_per_query: usize,     // Default: 10000
}
```

## Performance Targets

- **Latency:**
  - API response time: < 50ms (cached), < 200ms (DB query)
  - Worker collection: Non-blocking, doesn't affect API
  
- **Throughput:**
  - Worker: Handle 1000+ containers, collect stats every 5s
  - API: 1000+ requests/second (with caching)
  
- **Storage:**
  - Efficient compression for old data
  - Automatic retention/cleanup

## Migration Strategy

1. **Phase 1: Database Setup**
   - Install TimescaleDB extension
   - Convert existing tables to hypertables
   - Create indexes and continuous aggregates
   - Add retention policies

2. **Phase 2: Worker Refactoring**
   - Implement batch collection
   - Add batch insert logic
   - Implement image tracking
   - Add error handling and retries

3. **Phase 3: API Refactoring**
   - Switch from Docker API to database queries
   - Implement Redis caching (optional)
   - Add historical query endpoints
   - Optimize queries

4. **Phase 4: Testing & Optimization**
   - Load testing
   - Query optimization
   - Cache tuning
   - Monitor performance

## Error Handling

- **Worker:**
  - Retry failed Docker API calls (exponential backoff)
  - Log errors but continue running
  - Handle database connection failures gracefully
  - Queue failed inserts for retry

- **API:**
  - Fallback to database if cache unavailable
  - Return cached data if database query fails (stale data better than no data)
  - Proper error responses

## Monitoring & Observability

- **Metrics to Track:**
  - Worker: Collection rate, batch insert success rate, errors
  - API: Request latency, cache hit rate, query performance
  - Database: Query times, table sizes, compression ratio

- **Logging:**
  - Structured logging (JSON format)
  - Log levels: ERROR, WARN, INFO, DEBUG
  - Include correlation IDs for request tracing

## Future Enhancements

1. **Horizontal Scaling:**
   - Multiple worker instances (leader election)
   - Read replicas for database
   - Redis cluster for distributed caching

2. **Advanced Features:**
   - Alerting system
   - Anomaly detection
   - Cost tracking (resource usage)
   - Container lifecycle events

3. **Performance:**
   - GraphQL API for flexible queries
   - WebSocket for real-time updates
   - Data export/backup

