# Migration Notes

## What Changed

### Old Structure
```
backend/api/
├── src/          # Server + services + models
└── ui/           # Frontend
```

### New Structure
```
backend/
├── server/       # HTTP server only
├── services/     # Business logic + infrastructure
├── ui/           # Frontend
└── shared/       # Shared types
```

## Key Changes

1. **Separated concerns:**
   - Server only handles HTTP/routing
   - Services handle all business logic
   - Shared types in separate crate

2. **Workspace setup:**
   - Single `Cargo.toml` at `backend/` manages all crates
   - Shared dependencies defined once

3. **Import changes:**
   - Use `eyes_devine_shared` for models (not `crate::models`)
   - Use `eyes_devine_services` for services (not `crate::services`)

## Migration Checklist

- [x] Create workspace Cargo.toml
- [x] Create server/ crate
- [x] Create services/ crate
- [x] Create shared/ crate
- [x] Move ui/ to backend root
- [x] Update all imports
- [x] Update .cursorrules
- [ ] Test compilation
- [ ] Test running server
- [ ] Test UI build
- [ ] Remove old `backend/api/` folder (after confirming everything works)

## Next Steps

1. Test the build:
   ```bash
   cd backend
   cargo build
   ```

2. Build UI:
   ```bash
   cd backend/ui
   make build
   ```

3. Run server:
   ```bash
   cd backend/server
   cargo run
   ```

4. Once everything works, you can delete `backend/api/` folder

