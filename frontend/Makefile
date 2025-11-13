.PHONY: build dev watch clean serve

# Production build (for backend serving)
build:
	wasm-pack build --target web --out-dir pkg --release
	mkdir -p target/site
	cp index.html target/site/
	cp -r pkg target/site/pkg

# Development build (faster, larger binary)
dev:
	wasm-pack build --target web --out-dir pkg
	mkdir -p target/site
	cp index.html target/site/
	cp -r pkg target/site/pkg

# Hot reload development server with trunk
watch:
	trunk serve --port 4000

# Build with trunk (for production)
serve:
	trunk build --release

# Clean build artifacts
clean:
	rm -rf pkg target/site dist

