# TimescaleDB Setup Guide

## Using Docker Compose (Recommended)

The `docker-compose.yml` has been configured to use the official TimescaleDB Docker image, which includes PostgreSQL with TimescaleDB pre-installed.

### Start the Database

```bash
cd deploy
docker-compose up -d postgres
```

### Verify TimescaleDB is Installed

```bash
# Connect to the database
docker exec -it docker-monitor-postgres psql -U postgres -d docker_monitor

# Check if TimescaleDB extension is available
\dx timescaledb

# Or check version
SELECT extversion FROM pg_extension WHERE extname = 'timescaledb';
```

## Manual Setup (If using regular PostgreSQL)

If you're using a regular PostgreSQL image and want to install TimescaleDB manually:

### Option 1: Install TimescaleDB Extension in Existing Container

```bash
# Connect to PostgreSQL
docker exec -it docker-monitor-postgres psql -U postgres -d docker_monitor

# Install TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
```

**Note:** This requires the TimescaleDB binaries to be installed in the container, which the regular `postgres:18` image doesn't have by default.

### Option 2: Use TimescaleDB Docker Image (Recommended)

Simply use the `timescale/timescaledb` image as shown in the updated `docker-compose.yml`.

## Available TimescaleDB Images

- `timescale/timescaledb:latest-pg18` - Latest TimescaleDB with PostgreSQL 18
- `timescale/timescaledb:2.15.0-pg18` - Specific version
- `timescale/timescaledb:latest-pg16` - Latest with PostgreSQL 16
- `timescale/timescaledb:latest-pg15` - Latest with PostgreSQL 15

## After Setup

Once TimescaleDB is installed, run your migrations:

```bash
make migrate-up
```

The migrations will automatically:
1. Enable the TimescaleDB extension
2. Convert tables to hypertables
3. Create continuous aggregates
4. Set up retention policies
5. Enable compression

## Troubleshooting

### Check if TimescaleDB is available

```sql
SELECT * FROM pg_available_extensions WHERE name = 'timescaledb';
```

### Check if TimescaleDB is installed

```sql
SELECT * FROM pg_extension WHERE extname = 'timescaledb';
```

### View all hypertables

```sql
SELECT * FROM timescaledb_information.hypertables;
```

### View continuous aggregates

```sql
SELECT * FROM timescaledb_information.continuous_aggregates;
```

## Resources

- [TimescaleDB Documentation](https://docs.timescale.com/)
- [TimescaleDB Docker Hub](https://hub.docker.com/r/timescale/timescaledb)
- [Installation Guide](https://docs.timescale.com/install/latest/)

