# Building the Backend Docker Image

## Issue with Build Context

The `Cargo.lock` file is at the workspace root, not in `backend/`. This means you need to use the **workspace root** as the build context.

## Correct Build Command

### Option 1: Use Workspace Root as Build Context (Recommended)

```bash
# From workspace root
docker build -t eyes-devine:latest -f ./deploy/Dockerfile/api.Dockerfile .
```

**Note**: The build context is `.` (workspace root), not `./backend`

### Option 2: Use Backend Directory Dockerfile

If you want to use `./backend` as build context, use the Dockerfile in `backend/`:

```bash
# From workspace root
docker build -t eyes-devine:latest -f ./backend/Dockerfile ./backend
```

This Dockerfile handles `Cargo.lock` being optional.

## Why This Matters

- **Build context** determines what files Docker can access
- `Cargo.lock` is at workspace root for Rust workspaces
- If build context is `./backend`, you can't access `../Cargo.lock`
- Solution: Use workspace root as build context

## Quick Reference

```bash
# Recommended: Workspace root context
docker build -t eyes-devine:latest -f ./deploy/Dockerfile/api.Dockerfile .

# Alternative: Backend context (uses backend/Dockerfile)
docker build -t eyes-devine:latest -f ./backend/Dockerfile ./backend
```

## Running the Container

```bash
docker run -d \
  --name eyes-devine-server \
  -p 8080:8080 \
  -v /var/run/docker.sock:/var/run/docker.sock:ro \
  eyes-devine:latest
```

