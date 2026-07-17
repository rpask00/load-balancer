# Load Balancer

An HTTP load balancer written in Rust, with a live terminal dashboard (TUI) for
managing backend workers and switching balancing strategies on the fly.

The balancer accepts HTTP requests, picks a backend worker according to the
active strategy, and reverse-proxies the request to it. Backends ("workers") are
real OS child processes that the balancer spawns, monitors, and tears down at
runtime. 

<img width="1697" height="789" alt="Peek 2026-06-19 15-29" src="https://github.com/user-attachments/assets/35f4c6fa-81ed-4c5c-aa5f-37feda980a52" />


## Features

- **Reverse-proxy HTTP load balancing** built on `hyper` / `hyper-util` and
  `tokio`.
- **Pluggable balancing strategies** behind a trait + registry:
  - `Round Robin` — cycle through running workers in order.
  - `Least Connections` — pick the worker with the fewest in-flight requests
    (approximated via the worker's `Arc` reference count).
  - `Least Load` — pick the worker with the lowest connections-per-thread ratio.
- **Runtime strategy switching** via a `POST /strategy` HTTP endpoint or from the
  TUI.
- **Automatic strategy selection** via a rule-based `DecisionEngine` that
  periodically re-evaluates and applies the best strategy.
- **Dynamic worker pool** — spawn and shut down backend workers at runtime, each
  as a managed child process, with an automatic port pool.
- **Health checks & pruning** — background loops detect dead workers and reclaim
  their ports.
- **Interactive TUI dashboard** (`ratatui` + `crossterm`) with mouse and keyboard
  support for adding/removing workers and changing the active mode.
- **Load-generator tool** (`playground`) for firing concurrent traffic at the
  balancer.

## Architecture

```
                 ┌─────────────────────────────────────────────┐
   HTTP client   │                Load Balancer                 │
  ───────────►   │  127.0.0.1:1337 (hyper http1 server)         │
                 │                                              │
                 │   handle() ── strategy.select_worker() ──┐   │
                 │                                          │   │
                 │   ┌──────────────┐   ┌────────────────┐ │   │
                 │   │ Strategy     │   │ DecisionEngine │ │   │
                 │   │ Registry     │   │ (rule-based)   │ │   │
                 │   └──────────────┘   └────────────────┘ │   │
                 │   background loops: health-check, prune, │   │
                 │   auto-strategy, TUI dashboard           │   │
                 └──────────────────────────────────────────┼──┘
                                                             │ reverse proxy
                          ┌──────────────┬───────────────┬──┘
                          ▼              ▼               ▼
                    ┌──────────┐   ┌──────────┐    ┌──────────┐
                    │ lb_worker│   │ lb_worker│    │ lb_worker│   (child processes)
                    │  :3000   │   │  :3001   │ …  │  :30NN   │   axum servers
                    └──────────┘   └──────────┘    └──────────┘
```

The balancer holds a single shared `Arc<RwLock<LoadBalancer>>` that the HTTP
server, the TUI thread, and the background loops all coordinate through.

### Crate layout

| Path | Responsibility |
| --- | --- |
| `src/main.rs` | Entry point. Boots the HTTP server, TUI, and background loops (health check, pruning, auto-strategy). |
| `src/config.rs` | Static config: listen port, first worker port, max worker count. |
| `src/load_balancer/load_balancer.rs` | Core `LoadBalancer`: worker pool, port pool, request preparation, strategy management, lifecycle. |
| `src/load_balancer/worker.rs` | `Worker`: spawns/manages an `lb_worker` child process, proxies requests, tracks status. |
| `src/load_balancer/strategy/` | `LoadBalancingStrategy` trait, `LoadBalancingPolicy` enum, `StrategyRegistry`, and the three strategy implementations. |
| `src/load_balancer/decision_engine/` | `DecisionEngine` trait and rule-based engines (`Engine1`, `Engine2`) that auto-select a strategy. |
| `src/tui/` | Terminal dashboard: `App` state, components (main menu, add-worker popup, mode selector), and rendering. |
| `src/bin/lb_worker.rs` | The backend worker binary — a minimal `axum` server that echoes the request and simulates latency. |
| `src/bin/playground.rs` | A concurrent load generator that hammers the balancer with requests. |

### Binaries

This crate produces three binaries:

- **`load_balancer`** (default, `src/main.rs`) — the balancer + TUI.
- **`lb_worker`** (`src/bin/lb_worker.rs`) — a backend worker. The balancer
  launches these itself; you normally don't run it by hand.
- **`playground`** (`src/bin/playground.rs`) — a load-testing client.

## How it works

1. **Workers as child processes.** Each `Worker` shells out to
   `./target/debug/lb_worker` with `--port` and `--num-threads`. The worker is a
   small `axum` server that echoes the method/path it received and sleeps for a
   random interval to simulate work. The balancer talks to it over HTTP and
   shuts it down by writing `shutdown\n` to its stdin.
2. **Request handling.** For each incoming request, `handle()` reads the shared
   state, asks the active strategy to `select_worker()`, rewrites the URI to
   point at the chosen worker's port, and proxies the request through.
3. **Strategy selection.** Strategies implement `LoadBalancingStrategy` and are
   produced by a `StrategyRegistry` keyed on `LoadBalancingPolicy`. The "load"
   signals (connection counts) are approximated using each worker's `Arc`
   strong-reference count.
4. **Background loops.** Separate threads run a periodic health check (every
   ~5s), worker pruning + port reclamation (driven from the TUI loop, ~1s), and
   the decision engine that re-applies a strategy (every ~5s).
5. **TUI.** The dashboard renders the worker table and lets you add/delete
   workers and change the active mode using the keyboard or mouse.

> **Note:** On startup `main.rs` spawns 25 demo workers with random thread counts
> for demonstration. Adjust or remove that block in `src/main.rs` for your own
> use.

## Prerequisites

- A stable Rust toolchain — install via [rustup](https://rustup.rs/).
- A Unix-like OS. Worker shutdown uses Unix-specific process handling, so this
  targets Linux/macOS.

## Build

Build everything first — the balancer launches the **already-compiled**
`lb_worker` binary at `./target/debug/lb_worker`, so it must exist before you
start the balancer:

```bash
cargo build
```

## Run

Start the load balancer (this also opens the TUI dashboard):

```bash
cargo run --bin load_balancer
# or simply:
cargo run
```

The balancer listens on `http://127.0.0.1:1337` by default. Workers are assigned
ports starting at `3000`.

### Send traffic

In another terminal:

```bash
curl http://127.0.0.1:1337/test
```

Or use the bundled load generator to send concurrent batches:

```bash
cargo run --bin playground -- --iterations 100 --batch-size 5
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--iterations`, `-i` | `100` | Number of batches to send |
| `--batch-size`, `-b` | `5` | Concurrent requests per batch |

### Change the strategy at runtime

Via HTTP:

```bash
curl -X POST http://127.0.0.1:1337/strategy \
  -H 'Content-Type: application/json' \
  -d '{"strategy": "Round Robin"}'
```

Accepted strategy names: `Round Robin`, `Least Connections`, `Least Load`.

Or change it from the TUI (see below).

## TUI controls

| Key / Action | Effect |
| --- | --- |
| `↑` / `k`, `↓` / `j` | Move selection in the worker table |
| `a` / `A` | Open the "add worker" popup |
| `d` / `D` / `x` / `X` | Delete the selected worker |
| `q` / `Esc` | Quit |
| Mouse click | Select a row, press a button, or open the options menu |

**Add-worker popup:** type a name, `Tab` to switch to the port field, type a
numeric port, `Enter` to submit, `Esc` to cancel.

**Options / mode menu:** opens the strategy selector so you can switch between
Round Robin / Least Connections / Least Load.

## Configuration

Defaults live in `src/config.rs`:

| Constant | Default | Meaning |
| --- | --- | --- |
| `PORT` | `1337` | Port the balancer listens on |
| `FIRST_WORKER_PORT` | `3000` | First port assigned to spawned workers |
| `MAX_WORKERS_COUNT` | `1000` | Size of the worker port pool |

Logs are written to `tui.log` in the working directory (the TUI takes over the
terminal, so logging goes to a file).

## Extending

- **Add a balancing strategy:** create a type implementing
  `LoadBalancingStrategy` in `src/load_balancer/strategy/`, add a variant to
  `LoadBalancingPolicy`, and register it in `StrategyRegistry::default()`.
- **Add a decision engine:** implement the `DecisionEngine` trait (see
  `engine1.rs` / `engine2.rs`) with your own rules and wire it into the
  auto-strategy loop in `src/main.rs`.

## Tech stack

`tokio` · `hyper` / `hyper-util` · `axum` · `tower` · `ratatui` · `crossterm` ·
`clap` · `strum` · `serde` / `serde_json` · `color-eyre` · `reqwest` ·
`log` / `simplelog`
