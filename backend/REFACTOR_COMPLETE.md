# Refactoring Complete - Migration Guide

## New Structure

```
backend/
├── Cargo.toml              # Workspace manifest
├── server/                  # HTTP Server (Actix-web)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── handlers.rs
│       └── routes.rs
├── services/                # Business Logic + Infrastructure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── docker_service.rs
│       ├── cache_service.rs
│       ├── database.rs
│       └── entity/
├── ui/                      # Frontend (Leptos CSR)
│   ├── Cargo.toml
│   └── src/
└── shared/                  # Shared Types
    ├── Cargo.toml
    └── src/
        └── models.rs
```

## Next Steps

1. **Move UI folder:**
   - Move `backend/api/ui/` → `backend/ui/`
   - Update UI Cargo.toml to use shared crate
   - Update UI imports to use `eyes_devine_shared`

2. **Update UI to use shared models:**
   - Remove `ui/src/models.rs`
   - Update `ui/src/app.rs` to import from `eyes_devine_shared`

3. **Fix crate name imports:**
   - Use `eyes_devine_shared` (underscores) in Rust code
   - Crate name is `eyes-devine-shared` (hyphens) in Cargo.toml

4. **Test compilation:**
   - `cargo build` in backend/ should build all crates
   - Fix any import errors

5. **Clean up old files:**
   - Remove `backend/api/` after confirming everything works

