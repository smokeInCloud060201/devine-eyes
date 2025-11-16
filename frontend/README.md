# Devine Eyes - Frontend

React + TypeScript frontend for Docker monitoring application, built with Rspack (RsBuild).

## Tech Stack

- **React 18** - UI library
- **TypeScript** - Type safety
- **Rspack (RsBuild)** - Fast Rust-based bundler
- **Prettier** - Code formatting
- **ESLint** - Code linting

## Getting Started

### Prerequisites

- Node.js 18+ and npm

### Installation

```bash
# Install dependencies
npm install
# or
make install
```

### Development

```bash
# Start development server with hot reload (port 4000)
npm run dev
# or
make dev
```

The dev server will proxy API requests to `http://127.0.0.1:8080` (backend server).

### Building

```bash
# Production build
npm run build
# or
make build
```

### Code Quality

```bash
# Format code
npm run format
# or
make format

# Check formatting
npm run format:check
# or
make format-check

# Lint code
npm run lint
# or
make lint

# Type check
npm run type-check
# or
make type-check
```

### Preview Production Build

```bash
npm run preview
# or
make preview
```

## Project Structure

```
frontend/
├── src/
│   ├── pages/          # Page components
│   ├── components/     # Reusable components
│   ├── services/       # API services
│   ├── utils/          # Utility functions
│   ├── App.tsx         # Main app component
│   └── index.tsx       # Entry point
├── rsbuild.config.ts   # Rspack configuration
├── tsconfig.json       # TypeScript configuration
├── .prettierrc         # Prettier configuration
├── .eslintrc.cjs       # ESLint configuration
└── package.json        # Dependencies and scripts
```

## Configuration

### Rspack Configuration

The Rspack configuration is in `rsbuild.config.ts`. It includes:
- React plugin
- Sass support
- API proxy to backend (port 8080)
- Development server on port 4000

### TypeScript

TypeScript is configured with strict mode enabled. Path aliases are set up:
- `@/*` maps to `./src/*`

### Prettier

Prettier is configured with:
- Single quotes
- 2 spaces indentation
- 100 character line width
- Semicolons enabled

## Development Workflow

1. Start the backend server (port 8080)
2. Start the frontend dev server: `npm run dev`
3. Open `http://localhost:4000` in your browser
4. Make changes - hot reload will update automatically

## Building for Production

The production build outputs to the `dist/` directory. The backend server can serve these static files.

```bash
npm run build
# Output: dist/
```
