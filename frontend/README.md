# Frontend - Leptos UI

This is the Leptos frontend application for Eyes Devine, created using the Actix template pattern.

## Prerequisites

Install required tools:

1. **wasm-pack** - For building WebAssembly bundles:
```bash
cargo install wasm-pack
```

2. **trunk** - For hot reload development server (recommended):
```bash
cargo install trunk --locked
```

3. Ensure the `wasm32-unknown-unknown` target is installed:
```bash
rustup target add wasm32-unknown-unknown
```

## Development with Hot Reload

The easiest way to develop is using `trunk` with hot reload:

```bash
make watch
```

Or directly:
```bash
trunk serve
```

This will:
- Start a development server at `http://127.0.0.1:3000`
- Automatically rebuild on file changes
- Hot reload the browser when changes are detected
- Open the browser automatically

## Building

### Production Build (for backend serving)
```bash
make build
```

### Development Build (faster compilation, larger binary)
```bash
make dev
```

### Build with Trunk (alternative)
```bash
trunk build --release
```

## Development

The frontend is built using:
- **Leptos 0.8.0** - Reactive web framework
- **wasm-pack** - Build tool for WebAssembly
- **CSR Mode** - Client-Side Rendering (can be upgraded to SSR if needed)

## Structure

- `src/app.rs` - Main application component
- `src/lib.rs` - Entry point for WASM
- `index.html` - HTML template
- `style.css` - Stylesheet
- `leptos.toml` - Leptos configuration
- `Trunk.toml` - Trunk configuration for hot reload
- `pkg/` - Generated WASM/JS bundle (after wasm-pack build)
- `dist/` - Generated output from trunk (for hot reload)
- `target/site/` - Final build output (served by backend)

## Features

- ✅ Basic Leptos component structure
- ✅ Reactive signals and state management
- ✅ Hot reload development server with trunk
- ✅ Routing support (leptos_router included, can be added when needed)
- ✅ Modern CSS styling
- ✅ Ready for integration with backend API

## Integration with Backend

The backend server should serve the frontend from:
- `/pkg/*` - WASM and JS files
- `/` - HTML and static assets

Build the frontend before running the server:
```bash
make build
cd ../backend/server
cargo run
```

## Adding Routing

To add routing, import and use `leptos_router` components. The dependency is already included in `Cargo.toml`.

