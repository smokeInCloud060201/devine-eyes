# Eyes Devine Backend

Backend workspace for the Docker Monitor application.

## Structure

```
backend/
├── Cargo.toml          # Workspace manifest
├── server/             # HTTP Server (Actix-web)
├── services/           # Business Logic + Infrastructure
├── ui/                 # Frontend (Leptos CSR)
└── shared/             # Shared Types (API + UI)
```

## Crates

### `server/`
HTTP server using Actix-web. Handles:
- API routes (`/api/*`)
- Serving UI static files
- Request/response handling

### `services/`
Business logic and infrastructure:
- Docker service (container management)
- Cache service (Redis)
- Database connection
- Entity models (SeaORM)

### `ui/`
Leptos frontend (Client-Side Rendering):
- WASM-based UI
- Reactive components
- API client code

### `shared/`
Shared types between server and UI:
- Data models (ContainerInfo, ContainerStats, etc.)
- DTOs

## Building

### Build All Crates
```bash
cd backend
cargo build
```

### Build Individual Crate
```bash
cargo build -p eyes-devine-server
cargo build -p eyes-devine-services
cargo build -p eyes-devine-ui
cargo build -p eyes-devine-shared
```

## Running

### Build UI First
```bash
cd ui
make build
# or manually:
wasm-pack build --target web --out-dir pkg
mkdir -p target/site
cp index.html target/site/
cp -r pkg target/site/pkg
```

### Run Server
```bash
cd server
cargo run
```

The server will:
- Serve API at `/api/*`
- Serve UI files from `../ui/pkg/` and `../ui/target/site/`

## Development

### UI Development with Hot Reload
```bash
cd ui
make dev
```

Requires `cargo-watch`:
```bash
cargo install cargo-watch
```

### Server Development
```bash
cd server
cargo run
```

## Workspace Benefits

- Single dependency management
- Shared dependencies across crates
- Easier to build/test everything together
- Clear separation of concerns

