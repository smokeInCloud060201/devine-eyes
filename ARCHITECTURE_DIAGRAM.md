# Architecture Diagrams

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Docker Engine                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │Container1│  │Container2│  │Container3│  │  Image1  │       │
│  │  Stats   │  │  Stats   │  │  Stats   │  │ Metadata │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
└─────────────────────────────────────────────────────────────────┘
                            │
                            │ (bollard API)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Worker Service                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Collection Tasks (Async)                                │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │   │
│  │  │ Stats        │  │ Status       │  │ Images       │ │   │
│  │  │ (every 5s)   │  │ (every 30s)  │  │ (every 60s)  │ │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            │                                     │
│                            │ Batch (500-1000 records)            │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Batch Insert Service                                     │   │
│  │  - Collects metrics for 1-5 seconds                      │   │
│  │  - Batches into 500-1000 records                         │   │
│  │  - Single transaction insert                              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                            │
                            │ (SeaORM)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    TimescaleDB (PostgreSQL)                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Hypertables (Time-Series Optimized)                    │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │   │
│  │  │container_    │  │container_    │  │image_        │ │   │
│  │  │stats         │  │logs          │  │versions      │ │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Regular Tables                                          │   │
│  │  ┌──────────────┐  ┌──────────────┐                    │   │
│  │  │container_     │  │docker_       │                    │   │
│  │  │info           │  │images        │                    │   │
│  │  └──────────────┘  └──────────────┘                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Continuous Aggregates (Pre-computed)                   │   │
│  │  - Hourly stats                                         │   │
│  │  - Daily stats                                          │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                            ▲
                            │ (SeaORM queries)
                            │
┌─────────────────────────────────────────────────────────────────┐
│                      API Service                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Request Handler                                         │   │
│  │  1. Check Redis Cache                                   │   │
│  │  2. If miss: Query Database                             │   │
│  │  3. Store in cache                                       │   │
│  │  4. Return response                                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                            │                                     │
│                            │ (Optional)                          │
│                            ▼                                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Redis Cache                                             │   │
│  │  - containers:list (TTL: 10s)                           │   │
│  │  - stats:container:{id}:latest (TTL: 2s)                │   │
│  │  - image:{id} (TTL: 5m)                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                            │
                            │ (HTTP/SSE)
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Frontend (Leptos)                           │
│  - Container Dashboard                                           │
│  - Metrics Charts                                                │
│  - Image Browser                                                 │
│  - Log Viewer                                                    │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow: Collection (Worker)

```
┌─────────┐
│ Docker  │
│  API    │
└────┬────┘
     │
     │ 1. Poll containers
     ▼
┌─────────────────┐
│ Collection Task │
│ (Async Loop)    │
└────┬────────────┘
     │
     │ 2. Collect metrics
     ▼
┌─────────────────┐
│ Batch Buffer    │
│ (In-Memory)     │
└────┬────────────┘
     │
     │ 3. Accumulate (1-5s)
     │    or reach batch_size
     ▼
┌─────────────────┐
│ Batch Insert    │
│ (Transaction)   │
└────┬────────────┘
     │
     │ 4. Insert to DB
     ▼
┌─────────────────┐
│ TimescaleDB     │
│ (Hypertable)    │
└─────────────────┘
```

## Data Flow: Query (API)

```
┌─────────┐
│ Client  │
│ Request │
└────┬────┘
     │
     │ 1. HTTP Request
     ▼
┌─────────────────┐
│ API Handler     │
└────┬────────────┘
     │
     │ 2. Check Cache
     ▼
┌─────────────────┐      ┌──────────┐
│ Redis Cache     │      │ Cache   │
│                 │      │ Hit?    │
└────┬────────────┘      └────┬────┘
     │                        │
     │ No                     │ Yes
     ▼                        │
┌─────────────────┐          │
│ Query Database  │          │
│ (TimescaleDB)   │          │
└────┬────────────┘          │
     │                        │
     │ 3. Store in cache      │
     │    (if miss)           │
     │                        │
     └────────┬───────────────┘
              │
              │ 4. Return Response
              ▼
     ┌─────────────────┐
     │ JSON Response   │
     └─────────────────┘
```

## Time-Series Query Optimization

```
┌─────────────────────────────────────────────────────────┐
│  Query: Get stats for last 24 hours                    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ Check Time    │
                    │ Range         │
                    └───────┬───────┘
                            │
            ┌───────────────┼───────────────┐
            │               │               │
    < 1 hour       1-24 hours      > 24 hours
            │               │               │
            ▼               ▼               ▼
    ┌───────────┐  ┌───────────┐  ┌───────────┐
    │ Query     │  │ Query     │  │ Query     │
    │ Raw Data  │  │ Hourly    │  │ Daily     │
    │ (Fast)    │  │ Aggregate │  │ Aggregate │
    └───────────┘  └───────────┘  └───────────┘
```

## Batch Insert Strategy

```
Time: 0s ──────────────────────────────────────────> 5s

Worker collects metrics every 5 seconds:

┌─────────────────────────────────────────────────────┐
│ t=0s:  Container1 stats collected                    │
│ t=1s:  Container2 stats collected                   │
│ t=2s:  Container3 stats collected                   │
│ t=3s:  Container1 stats collected (again)          │
│ t=4s:  Container2 stats collected (again)           │
│ t=5s:  Batch timeout OR batch_size reached          │
│        → Insert all collected stats in one          │
│          transaction (e.g., 500 records)            │
└─────────────────────────────────────────────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ Single INSERT │
                    │ (500 records) │
                    └───────────────┘
                            │
                            ▼
                    ┌───────────────┐
                    │ TimescaleDB   │
                    │ (Efficient)   │
                    └───────────────┘
```

## Component Responsibilities

### Worker Service
```
┌─────────────────────────────────────────┐
│  Worker Service Responsibilities       │
├─────────────────────────────────────────┤
│ ✓ Poll Docker API                      │
│ ✓ Collect metrics/stats/images          │
│ ✓ Batch data                            │
│ ✓ Insert to database                    │
│ ✓ Handle errors gracefully              │
│ ✓ Never block API                       │
│ ✗ Never serve requests                  │
└─────────────────────────────────────────┘
```

### API Service
```
┌─────────────────────────────────────────┐
│  API Service Responsibilities            │
├─────────────────────────────────────────┤
│ ✓ Serve HTTP requests                   │
│ ✓ Query database                        │
│ ✓ Cache responses                       │
│ ✓ Handle pagination                     │
│ ✓ Optimize queries                      │
│ ✗ Never query Docker directly           │
│ ✗ Never block on worker                 │
└─────────────────────────────────────────┘
```

### Database (TimescaleDB)
```
┌─────────────────────────────────────────┐
│  Database Responsibilities              │
├─────────────────────────────────────────┤
│ ✓ Store time-series data                │
│ ✓ Optimize time-range queries           │
│ ✓ Compress old data                     │
│ ✓ Auto-retention (cleanup)              │
│ ✓ Pre-compute aggregates                │
│ ✓ Index for fast lookups                │
└─────────────────────────────────────────┘
```

## Error Handling Flow

```
┌─────────────────┐
│ Worker Error    │
└────┬────────────┘
     │
     ├─ Docker API Error
     │  └─> Retry (exponential backoff)
     │     └─> Log error, continue
     │
     ├─ Database Error
     │  └─> Retry (exponential backoff)
     │     └─> Queue for later retry
     │
     └─ Fatal Error
        └─> Log error, restart worker
```

```
┌─────────────────┐
│ API Error       │
└────┬────────────┘
     │
     ├─ Cache Error
     │  └─> Fallback to database
     │
     ├─ Database Error
     │  └─> Return cached data (if available)
     │     └─> Or return error response
     │
     └─ Query Timeout
        └─> Return error, log for investigation
```

