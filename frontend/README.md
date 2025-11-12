# Eyes Devine Frontend

Leptos SSR frontend application for Docker Monitor.

## Quick Start

### Using Makefile (Recommended)

1. **Install dependencies:**
```bash
make install
```

2. **Development mode with hot reload:**
```bash
make dev
```

3. **Production build and run:**
```bash
make run
```

### Manual Setup

1. Install dependencies:
```bash
rustup target add wasm32-unknown-unknown
```

2. Build the frontend:
```bash
cargo build --release
```

3. Build the WASM bundle:
```bash
wasm-pack build --target web --out-dir pkg --release
```

4. Copy files to site directory:
```bash
mkdir -p target/site
cp index.html target/site/
cp -r pkg target/site/
```

5. Run the server:
```bash
cargo run --release --bin server
```

## Makefile Commands

| Command | Description |
|---------|-------------|
| `make install` | Install required dependencies |
| `make build` | Build everything (server + WASM, release mode) |
| `make build-wasm` | Build only WASM bundle (release) |
| `make build-wasm-dev` | Build only WASM bundle (dev, faster) |
| `make build-server` | Build only server binary (release) |
| `make build-server-dev` | Build only server binary (dev) |
| `make run` | Build and run server (production) |
| `make run-dev` | Build and run server (dev mode) |
| `make dev` | **Development mode with hot reload (Rust only)** |
| `make dev-full` | Development mode with full hot reload (Rust + WASM) |
| `make clean` | Clean all build artifacts |
| `make help` | Show all available commands |

## Development Workflow

### Recommended: Fast Development
```bash
make dev
```
- Watches Rust code changes and auto-reloads server
- WASM needs manual rebuild: `make build-wasm-dev` when UI code changes
- Fastest option for backend/server development

### Full Hot Reload
```bash
make dev-full
```
- Watches Rust code and automatically rebuilds WASM
- Slower but fully automatic
- Best for frontend/UI development

### Prerequisites for Hot Reload
```bash
cargo install cargo-watch
```

## Running

Set the backend API URL (optional, defaults to http://127.0.0.1:8080):
```bash
export BACKEND_API_URL=http://127.0.0.1:8080
```

Set the frontend port (optional, defaults to 3000):
```bash
export PORT=3000
```

The frontend will be available at http://127.0.0.1:3000

## Architecture

- **Server**: Axum-based HTTP server with Leptos SSR
- **Client**: WASM bundle that hydrates the server-rendered HTML
- **API**: Calls backend REST API at `http://127.0.0.1:8080/api/*`

## Environment Variables

- `BACKEND_API_URL`: Backend API base URL (default: http://127.0.0.1:8080)
- `PORT`: Frontend server port (default: 3000)

## Project Structure

```
frontend/
├── src/
│   ├── main.rs      # Axum server entry point
│   ├── app.rs       # Leptos UI components
│   ├── lib.rs       # Library exports
│   └── style.css    # Styles
├── Cargo.toml       # Rust dependencies
├── leptos.toml      # Leptos configuration
├── index.html       # HTML template
├── Makefile         # Build automation
└── README.md        # This file
```

