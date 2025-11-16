# Fix PostgreSQL Permission Issues

If you're getting "Permission denied" errors when starting the PostgreSQL container, follow these steps:

## Solution 1: Remove and Recreate the Volume (Recommended)

This is the cleanest solution if you don't have important data:

```bash
cd deploy

# Stop containers
docker-compose down

# Remove the postgres volume
docker volume rm deploy_postgres_data

# Start containers again
docker-compose up -d postgres
```

## Solution 2: Fix Permissions on Existing Volume

If you have data you want to keep:

```bash
cd deploy

# Stop containers
docker-compose down

# Fix permissions using a temporary container
docker run --rm \
  -v deploy_postgres_data:/data \
  -u root \
  timescale/timescaledb:latest-pg18 \
  chown -R postgres:postgres /data

# Start containers again
docker-compose up -d postgres
```

## Solution 3: Use Named Volume with Proper Permissions

If the above doesn't work, you can manually create the volume:

```bash
# Create volume
docker volume create --name deploy_postgres_data

# Set permissions
docker run --rm \
  -v deploy_postgres_data:/data \
  -u root \
  timescale/timescaledb:latest-pg18 \
  sh -c "mkdir -p /data && chown -R postgres:postgres /data"

# Start containers
cd deploy
docker-compose up -d postgres
```

## Why This Happens

The permission error occurs because:
1. The Docker volume was created with different ownership
2. The TimescaleDB container runs as the `postgres` user (UID 999)
3. The volume directory doesn't have the correct permissions for this user

## Verify It's Working

After fixing, verify the container starts correctly:

```bash
# Check container logs
docker logs docker-monitor-postgres

# Should see messages like:
# "database system is ready to accept connections"
# "TimescaleDB extension loaded successfully"
```

## Prevention

The updated `docker-compose.yml` removes the custom `PGDATA` setting, which lets PostgreSQL handle directory creation automatically with the correct permissions.

