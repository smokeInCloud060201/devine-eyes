# System Design Summary - Key Decisions

## Architecture Overview

### Current State
- ✅ API service exists but queries Docker directly (blocking, low throughput)
- ✅ Worker service exists but only collects container info (not metrics/images)
- ✅ Database exists but not optimized for time-series
- ❌ No image tracking
- ❌ No caching layer
- ❌ No batch processing

### Target State
- ✅ Worker: Collects metrics, status, images → stores in TimescaleDB
- ✅ API: Queries database only (never blocks on Docker)
- ✅ TimescaleDB: Optimized for time-series queries
- ✅ Redis: Optional caching for low latency
- ✅ Batch inserts: High throughput

## Key Design Decisions

### 1. Database: TimescaleDB (PostgreSQL Extension)
**Why?**
- Built on PostgreSQL (familiar, compatible with SeaORM)
- Automatic partitioning by time (hypertables)
- Continuous aggregates for pre-computed metrics
- Compression and retention policies
- Excellent query performance for time-range queries

**Alternatives Considered:**
- InfluxDB: More complex, different query language
- ClickHouse: Overkill for this use case
- Plain PostgreSQL: Works but needs manual partitioning

### 2. Separation: Worker vs API
**Why?**
- Worker never blocks API requests
- Can scale independently
- Worker can fail without affecting API (serves stale data)
- Clear separation of concerns

**Data Flow:**
```
Docker → Worker → Database → API → Frontend
```

### 3. Batch Processing
**Why?**
- Reduces database round-trips
- Better throughput (100-1000 inserts per batch)
- Reduces connection pool pressure
- TimescaleDB handles batch inserts efficiently

**Implementation:**
- Collect metrics for 1-5 seconds
- Batch into 500-1000 records
- Single insert transaction

### 4. Caching Strategy (Redis - Optional)
**Why?**
- Reduces database load
- Sub-50ms response times for cached data
- Can serve stale data if database is slow

**Cache TTLs:**
- Container list: 10s (changes infrequently)
- Latest stats: 2s (needs to be fresh)
- Image metadata: 5min (rarely changes)
- Historical queries: 30s (expensive queries)

### 5. Collection Intervals
**Why?**
- Balance between freshness and resource usage
- Different intervals for different data types

**Recommended:**
- Container stats: 5-10 seconds (high frequency)
- Container status: 30 seconds (changes less often)
- Images: 60 seconds (rarely changes)
- Logs: Real-time streaming (optional)

## Database Schema Changes

### New Tables
1. **`docker_images`** - Track image metadata
2. **`image_versions`** - Time-series of image changes

### Table Conversions
1. **`container_stats`** → Hypertable (time-series optimized)
2. **`container_logs`** → Hypertable (time-series optimized)
3. **`container_info`** → Keep as regular table (snapshot data)

### New Features
- Continuous aggregates (pre-computed hourly/daily stats)
- Retention policies (auto-cleanup old data)
- Compression (reduce storage for old data)

## Implementation Phases

### Phase 1: Database Setup (Foundation)
**Priority: HIGH**
- Install TimescaleDB extension
- Convert tables to hypertables
- Create indexes
- Add retention policies
- Create continuous aggregates

**Estimated Time: 2-4 hours**

### Phase 2: Worker Enhancement (Collection)
**Priority: HIGH**
- Implement batch collection
- Add metrics collection (stats)
- Add image tracking
- Implement batch inserts
- Add error handling

**Estimated Time: 4-6 hours**

### Phase 3: API Refactoring (Serving)
**Priority: HIGH**
- Switch from Docker API to database queries
- Add historical query endpoints
- Implement query optimization
- Add pagination/filtering

**Estimated Time: 3-5 hours**

### Phase 4: Caching (Performance)
**Priority: MEDIUM**
- Add Redis integration
- Implement cache layer
- Add cache invalidation
- Monitor cache hit rates

**Estimated Time: 2-3 hours**

### Phase 5: Testing & Optimization
**Priority: MEDIUM**
- Load testing
- Query optimization
- Performance tuning
- Documentation

**Estimated Time: 2-4 hours**

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| API Response (cached) | < 50ms | N/A |
| API Response (DB query) | < 200ms | ~500ms+ |
| Worker collection | Non-blocking | Blocking |
| Throughput (API) | 1000+ req/s | ~100 req/s |
| Database inserts | 10k+ records/s | ~100 records/s |

## Technology Stack

### Current
- **Database**: PostgreSQL (via SeaORM)
- **Worker**: Rust (tokio)
- **API**: actix-web
- **Docker**: bollard

### Additions
- **TimescaleDB**: PostgreSQL extension
- **Redis**: Optional caching (redis-rs crate)
- **Batch Processing**: Custom implementation

## Configuration Changes

### Worker Config
```toml
[worker]
stats_interval = "5s"
status_interval = "30s"
image_interval = "60s"
batch_size = 500
batch_timeout = "1s"
```

### API Config
```toml
[api]
cache_ttl_containers = "10s"
cache_ttl_stats = "2s"
cache_ttl_images = "5m"
max_query_range_days = 30
max_results_per_query = 10000
```

## Migration Checklist

### Database
- [ ] Install TimescaleDB extension
- [ ] Convert `container_stats` to hypertable
- [ ] Convert `container_logs` to hypertable
- [ ] Create `docker_images` table
- [ ] Create `image_versions` hypertable
- [ ] Create indexes
- [ ] Create continuous aggregates
- [ ] Add retention policies
- [ ] Test queries

### Worker
- [ ] Implement batch collection
- [ ] Add stats collection
- [ ] Add image collection
- [ ] Implement batch inserts
- [ ] Add error handling
- [ ] Add configuration
- [ ] Test collection

### API
- [ ] Refactor to use database
- [ ] Add historical endpoints
- [ ] Add query optimization
- [ ] Add pagination
- [ ] Test endpoints

### Optional (Caching)
- [ ] Add Redis dependency
- [ ] Implement cache layer
- [ ] Add cache invalidation
- [ ] Monitor cache performance

## Risks & Mitigations

### Risk 1: Database Performance
**Mitigation:**
- Use TimescaleDB continuous aggregates
- Proper indexing
- Query optimization
- Consider read replicas if needed

### Risk 2: Worker Lag
**Mitigation:**
- Batch processing
- Connection pooling
- Async processing
- Error handling

### Risk 3: Cache Invalidation
**Mitigation:**
- Short TTLs
- Event-based invalidation (future)
- Fallback to database

### Risk 4: Data Loss
**Mitigation:**
- Retry logic
- Error logging
- Monitoring
- Backup strategy

## Success Criteria

1. ✅ API never blocks on Docker API calls
2. ✅ Worker collects data without affecting API
3. ✅ Historical queries are fast (< 200ms)
4. ✅ System handles 100+ containers smoothly
5. ✅ Data retention works automatically
6. ✅ Image tracking works correctly

## Next Steps

1. **Review this design** with team/stakeholders
2. **Approve approach** before implementation
3. **Start with Phase 1** (Database setup)
4. **Iterate** through phases
5. **Test** at each phase
6. **Deploy** incrementally

## Questions to Consider

1. **Redis Required?** 
   - Can start without it, add later if needed
   - Recommended for production

2. **Collection Frequency?**
   - Start with 5s for stats, adjust based on load
   - Can be configured per environment

3. **Data Retention?**
   - Start with 7 days raw, 30 days aggregated
   - Adjust based on storage capacity

4. **Image Tracking?**
   - Essential for full monitoring
   - Can be added in Phase 2

5. **Logs Storage?**
   - Currently optional
   - Can be expensive (large volume)
   - Consider log rotation/retention

