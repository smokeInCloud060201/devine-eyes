# Docker Monitor - Eyes Devine

A web application to monitor Docker containers locally, built with Rust, Actix-web, and PostgreSQL.

## Features

- **Container Monitoring**: Track total Docker resource usage (CPU, RAM, memory, network)
- **Per-Container Stats**: Monitor individual container metrics
- **Log Tracing**: View and filter logs from each container
- **Real-time Updates**: Auto-refreshing dashboard (every 5 seconds)

## Tech Stack

- **Backend**: Rust with Actix-web framework
- **Database**: PostgreSQL 18 with SeaORM (optional)
- **Cache**: Redis (optional)
- **Docker API**: Bollard crate
- **Frontend**: Modern HTML/CSS/JavaScript

## Prerequisites

- Rust (latest stable version)
- Docker Desktop or Docker Engine running locally
- Docker Compose (for database setup)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd eyes-devine
```

2. Create a `.env` file:
```bash
cp .env.example .env
```

3. Update the `.env` file with your configuration:
```env
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/docker_monitor
REDIS_URL=redis://localhost:6379
SERVER_HOST=127.0.0.1
SERVER_PORT=8080
```

4. Start PostgreSQL and Redis using Docker Compose:
```bash
docker-compose up -d
```

Or use the Makefile:
```bash
make docker-up
```

5. Run database migrations:
```bash
cd migrations && cargo run -- up
```

Or use the Makefile:
```bash
make migrate-up
```

6. Build and run the application:
```bash
cargo build --release
cargo run
```

Or for development:
```bash
cargo run
```

7. Open your browser and navigate to:
```
http://127.0.0.1:8080
```

## Database Setup

The project includes a `docker-compose.yml` file that sets up:
- **PostgreSQL 18**: Database server on port 5432
- **Redis 7**: Cache server on port 6379

### Using Docker Compose

Start services:
```bash
docker-compose up -d
```

Stop services:
```bash
docker-compose down
```

### Database Migrations

SeaORM migrations are located in the `migrations/` directory.

**Run migrations:**
```bash
cd migrations && cargo run -- up
```

**Rollback last migration:**
```bash
cd migrations && cargo run -- down
```

**Fresh migration (drop all and re-run):**
```bash
cd migrations && cargo run -- fresh
```

**Check migration status:**
```bash
cd migrations && cargo run -- status
```

Or use the Makefile commands:
- `make migrate-up` - Run migrations
- `make migrate-down` - Rollback migration
- `make migrate-fresh` - Fresh migration
- `make migrate-status` - Check status

## API Endpoints

- `GET /` - Web dashboard
- `GET /api/stats/total` - Get total Docker statistics
- `GET /api/containers` - List all containers
- `GET /api/containers/stats` - Get stats for all containers
- `GET /api/containers/{id}/stats` - Get stats for a specific container
- `GET /api/containers/{id}/logs?limit={n}` - Get logs for a specific container

## Usage

1. **View Total Stats**: The dashboard shows aggregated statistics for all containers
2. **View Container Stats**: Click on any container card to see detailed metrics
3. **View Logs**: 
   - Select a container from the dropdown
   - Set a log limit (default: 100)
   - Click "Load Logs" to view container logs

## Development

The application structure:
```
src/
├── main.rs           # Application entry point
├── config.rs         # Configuration management
├── database.rs       # Database setup with SeaORM
├── entity/           # SeaORM entities
│   ├── container_stats.rs
│   └── container_logs.rs
├── handlers.rs       # API route handlers
├── models.rs         # Data models (DTOs)
└── services/
    ├── docker_service.rs  # Docker API integration
    └── cache_service.rs   # Redis caching

migrations/
├── src/
│   ├── lib.rs        # Migration registry
│   ├── main.rs       # Migration CLI
│   └── m*.rs         # Migration files
└── Cargo.toml        # Migration crate config
```

## Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (default: `postgresql://postgres:postgres@localhost:5432/docker_monitor`)
- `REDIS_URL`: Redis connection string (optional)
- `SERVER_HOST`: Server host (default: 127.0.0.1)
- `SERVER_PORT`: Server port (default: 8080)

## Notes

- The application connects to the local Docker daemon
- Make sure Docker is running before starting the application
- The database and Redis are optional - the app will work without them
- Logs are fetched directly from Docker, not stored in the database (yet)
- Database migrations must be run before starting the application if using PostgreSQL

## Future Enhancements

- Historical data storage and visualization
- Real-time log streaming
- Container actions (start/stop/restart)
- Alerting and notifications
- Multi-host Docker monitoring
- Performance metrics over time

## License

[Your License Here]
