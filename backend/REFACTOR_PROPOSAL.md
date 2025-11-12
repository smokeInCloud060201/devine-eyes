# Backend Refactoring Proposal

## Recommended Structure (CSR + DDD)

Since you're using **Client-Side Rendering (CSR)** with Leptos, here's a cleaner structure:

```
backend/
├── Cargo.toml                    # Workspace manifest
│
├── server/                       # HTTP Server (Actix-web)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs              # Server entry point
│       ├── config.rs            # Server configuration
│       ├── routes.rs            # Route definitions
│       └── middleware.rs        # Auth, CORS, etc.
│
├── domain/                       # Domain Layer (DDD - Business Logic)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── container/           # Container domain
│       │   ├── mod.rs
│       │   ├── entity.rs        # Domain entities
│       │   ├── repository.rs    # Repository trait
│       │   └── service.rs        # Domain services
│       └── stats/               # Stats domain
│           ├── mod.rs
│           ├── entity.rs
│           └── service.rs
│
├── infrastructure/               # Infrastructure Layer (External Services)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── docker/              # Docker client implementation
│       │   ├── mod.rs
│       │   └── client.rs
│       ├── cache/                # Redis cache implementation
│       │   ├── mod.rs
│       │   └── redis_client.rs
│       ├── database/             # Database implementation
│       │   ├── mod.rs
│       │   ├── connection.rs
│       │   └── repositories/    # Repository implementations
│       │       ├── container_repository.rs
│       │       └── stats_repository.rs
│       └── migrations/          # Database migrations
│           └── ...
│
├── application/                  # Application Layer (Use Cases)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── handlers/             # HTTP handlers (thin layer)
│       │   ├── mod.rs
│       │   ├── container_handler.rs
│       │   └── stats_handler.rs
│       └── dto/                  # Data Transfer Objects
│           ├── mod.rs
│           └── container_dto.rs
│
├── ui/                           # Frontend (Leptos CSR)
│   ├── Cargo.toml
│   ├── index.html
│   └── src/
│       ├── lib.rs
│       ├── app.rs
│       ├── components/
│       └── models.rs
│
└── shared/                       # Shared Types (API + UI)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── models.rs             # Shared DTOs/models
```

## Key Improvements Over Your Proposal:

### ✅ **Clear Separation of Concerns**
- **server/** - Only HTTP/web server concerns
- **domain/** - Pure business logic (no dependencies on infrastructure)
- **infrastructure/** - External services (Docker, Redis, Database)
- **application/** - Use cases/orchestration
- **ui/** - Frontend (CSR)
- **shared/** - Types shared between API and UI

### ✅ **DDD Principles**
- Domain layer is independent (can be tested without infrastructure)
- Infrastructure implements domain interfaces
- Application layer orchestrates domain + infrastructure

### ✅ **Workspace Benefits**
- Single `Cargo.toml` at root manages all crates
- Shared dependencies
- Easier to build/test everything together

### ✅ **Simpler Than Your Proposal**
- No confusion between SSR/CSR (you're using CSR)
- No `api.rs` in server (routes are in `routes.rs`)
- Clear naming: `domain/` not `api/` for business logic

## Alternative: Simpler Structure (If DDD is Overkill)

If DDD feels too complex for your current needs:

```
backend/
├── Cargo.toml                    # Workspace
│
├── server/                       # HTTP Server
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── handlers.rs           # HTTP handlers
│       └── routes.rs
│
├── services/                     # Business Logic + Infrastructure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── docker_service.rs
│       ├── cache_service.rs
│       └── database/
│           ├── mod.rs
│           └── migrations/
│
├── ui/                           # Frontend
│   └── ...
│
└── shared/                       # Shared types
    └── ...
```

## Recommendation

**Start with the simpler structure**, then refactor to DDD if:
- The project grows significantly
- You need complex business rules
- Multiple teams are working on it
- You need to swap infrastructure easily

## Migration Path

1. Create workspace `Cargo.toml`
2. Move current `api/` → `server/`
3. Extract services → `services/` or `domain/` + `infrastructure/`
4. Keep `ui/` as is
5. Create `shared/` for common types

Would you like me to implement one of these structures?

