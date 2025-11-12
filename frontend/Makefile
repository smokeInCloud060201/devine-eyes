.PHONY: build clean install dev run build-wasm build-server setup-site

# Detect OS for cross-platform compatibility
ifeq ($(OS),Windows_NT)
    CP = copy
    MKDIR = if not exist
    RM = del /s /q
    RMDIR = rmdir /s /q
    SEP = \\
else
    CP = cp
    MKDIR = mkdir -p
    RM = rm -rf
    RMDIR = rm -rf
    SEP = /
endif

# Install dependencies
install:
	@echo "Installing dependencies..."
	@rustup target add wasm32-unknown-unknown
	@echo "Dependencies installed!"

# Setup site directory
setup-site:
	@echo "Preparing site directory..."
ifeq ($(OS),Windows_NT)
	@if not exist target\site mkdir target\site
	@copy index.html target\site\ 2>nul || echo.
	@xcopy /E /I /Y pkg target\site\pkg\ 2>nul || echo.
else
	@mkdir -p target/site
	@cp index.html target/site/
	@cp -r pkg target/site/
endif

# Build WASM bundle for client-side
build-wasm:
	@echo "Building WASM bundle..."
	@wasm-pack build --target web --out-dir pkg --release -- --no-default-features
	@echo "WASM build complete!"

# Build WASM bundle for development (faster, no optimization)
build-wasm-dev:
	@echo "Building WASM bundle (dev mode)..."
	@wasm-pack build --target web --out-dir pkg -- --no-default-features
	@$(MAKE) setup-site
	@echo "WASM dev build complete!"

# Build server binary
build-server:
	@echo "Building server..."
	@cargo build --release --bin server --features ssr
	@echo "Server build complete!"

# Build server binary (dev mode)
build-server-dev:
	@echo "Building server (dev mode)..."
	@cargo build --bin server --features ssr
	@echo "Server dev build complete!"

# Build everything
build: install build-server build-wasm
	@$(MAKE) setup-site
	@echo "Build complete! Files are in target/site/"

# Run the server (production mode)
run: build
	@echo "Starting frontend server..."
	@cargo run --release --bin server --features ssr

# Run the server (dev mode, no optimization)
run-dev: build-server-dev build-wasm-dev
	@echo "Starting frontend server (dev mode)..."
	@cargo run --bin server --features ssr

# Development mode with hot reload
dev:
	@echo "Starting development mode with hot reload..."
	@echo "Make sure cargo-watch is installed: cargo install cargo-watch"
	@echo "Building initial WASM bundle..."
	@$(MAKE) build-wasm-dev
	@echo "Starting server with watch mode..."
	@echo "Server will auto-reload on Rust code changes."
	@echo "WASM will need manual rebuild: make build-wasm-dev"
	@cargo watch -x "run --bin server --features ssr"

# Development mode with full hot reload (watches both Rust and rebuilds WASM)
dev-full:
	@echo "Starting development mode with full hot reload..."
	@echo "Make sure cargo-watch is installed: cargo install cargo-watch"
	@echo "Building initial WASM bundle..."
	@$(MAKE) build-wasm-dev
	@echo "Starting server with watch mode (will rebuild WASM on Rust changes)..."
ifeq ($(OS),Windows_NT)
	@cargo watch -x "run --bin server --features ssr" -s "wasm-pack build --target web --out-dir pkg -- --no-default-features && xcopy /E /I /Y pkg target\\site\\pkg\\"
else
	@cargo watch -x "run --bin server --features ssr" -s "wasm-pack build --target web --out-dir pkg -- --no-default-features && cp -r pkg target/site/"
endif

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
ifeq ($(OS),Windows_NT)
	@if exist pkg rmdir /s /q pkg
	@if exist target\site rmdir /s /q target\site
	@if exist target\debug rmdir /s /q target\debug
	@if exist target\release rmdir /s /q target\release
else
	@rm -rf pkg target/site target/debug target/release
endif
	@echo "Clean complete!"

# Help target
help:
	@echo "Available targets:"
	@echo "  make install      - Install required dependencies"
	@echo "  make build        - Build everything (server + WASM, release mode)"
	@echo "  make build-wasm   - Build only WASM bundle (release)"
	@echo "  make build-wasm-dev - Build only WASM bundle (dev, faster)"
	@echo "  make build-server - Build only server binary (release)"
	@echo "  make build-server-dev - Build only server binary (dev)"
	@echo "  make run          - Build and run server (production)"
	@echo "  make run-dev      - Build and run server (dev mode)"
	@echo "  make dev          - Run in development mode with hot reload (Rust only)"
	@echo "  make dev-full     - Run in development mode with full hot reload (Rust + WASM)"
	@echo "  make clean        - Clean all build artifacts"
	@echo "  make help         - Show this help message"
	@echo ""
	@echo "Recommended for development:"
	@echo "  make dev          - Fastest, watches Rust code only"
	@echo "  make dev-full     - Slower, but rebuilds WASM automatically"

