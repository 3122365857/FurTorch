# TorchRust Web Application Architecture

## Overview

TorchRust is a Torchlight Infinite companion app that tracks drops, income, and map statistics. The web version is organized as a monorepo with:

- **Rust** – Core logic (log parsing, price calculation, drop processing)
- **Node.js** – API server (Express)
- **React** – Frontend UI (Vite + TypeScript)

## Folder Structure

```
TorchRust/
├── crates/
│   └── torchrust-core/    # Rust core logic
│       ├── src/
│       │   ├── config.rs      # Config types
│       │   ├── log_parser.rs  # Structured log → JSON
│       │   ├── price.rs       # Price/tax logic
│       │   └── drop.rs        # Drop scanning & processing
│       └── Cargo.toml
├── server/                # Node.js API
│   ├── src/
│   │   └── index.ts       # Express routes
│   └── package.json
├── web/                   # React frontend
│   ├── src/
│   │   ├── features/
│   │   │   └── dashboard/
│   │   └── App.tsx
│   └── package.json
├── data/                  # Runtime & config data
├── tmp/                   # Debug: UE_game*.log backups for use_tmp_log
│   ├── config.json        # User config (locale, cost, opacity, tax, etc.)
│   └── ttd_price.json     # Item price table (Korean)
├── legacy/                # Original Python/Tk desktop app (reference)
│   ├── index.py
│   ├── font/, json/, pet/
│   └── README.md
├── docs/                  # Documentation
│   └── *.txt
├── Cargo.toml             # Rust workspace
└── package.json
```

## Data Flow

1. **Config** – `data/config.json` is read/written by the Node.js server via `/api/config`. Default locale: `ko`.
2. **Price table** – `data/ttd_price.json` (Korean names) is served via `/api/price-table` and `/api/full-table`.
3. **Log parsing** – Rust `torchrust-core` parses game log text and produces structured drop data.
4. **UI** – React fetches config and stats from the API and displays them.

## API Endpoints

- `GET /api/config` – Read config (locale defaults to `ko`)
- `PATCH /api/config` – Update config
- `GET /api/price-table` – Item price table (ttd_price.json, Korean)
- `POST /api/process-drops` – Process log chunk (body: `{ log_text }`), returns updated stats
- `GET /api/stats` – Current drop/income state
- `POST /api/stats/reset` – Reset stats
- `GET /api/health` – Health check
- `GET /api/log-watcher` – Log watcher status (path, ok, error)

## Log Path Resolution

Log path is resolved in this order:

1. **`log_path`** – Manual override (absolute or relative to project)
2. **`use_tmp_log`** – Debug mode: use `tmp/` folder. Picks most recent `UE_game*.log`
3. **Dynamic** – Windows: find Torchlight process via PowerShell, derive path from exe. Linux: try Steam paths.

## Log Watcher

- Polls game log every `log_poll_interval_sec` (1–10, default 1)
- Reads new content and processes via `torchrust-cli`
- Config: `log_path`, `use_tmp_log`, `log_poll_interval_sec`, `log_watcher_enabled`

## Running the Application

### Option A: Rust server (single binary, recommended)

```bash
npm run build:web          # Build frontend
cargo run -p torchrust-server
# or: npm run dev:rust
```

- App: http://localhost:3001 (serves API + static frontend)

### Option B: Node.js dev (separate processes)

```bash
# Terminal 1: API server
cd server && npm run dev

# Terminal 2: React frontend
cd web && npm run dev
```

- Frontend: http://localhost:5173
- API: http://localhost:3001

## Rust Server (torchrust-server)

- Axum HTTP server, uses `torchrust-core` directly (no subprocess)
- Log path resolution and log watcher in Rust
- Serves `web/dist` as static files
- Build: `cargo build -p torchrust-server --release`

## Rust CLI (torchrust-cli, optional)

- Reads JSON from stdin, writes to stdout
- Used by Node.js server via `child_process.spawn`
- Build: `cargo build -p torchrust-cli` (release or debug)

## Next Steps

- [ ] Add WebSocket or polling for real-time stats
