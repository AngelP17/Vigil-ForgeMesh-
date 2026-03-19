# Vigil

**Operational Incident Intelligence Platform**

![Vigil landing page (dark theme)](docs/landing-page.png)

Evolved from ForgeMesh distributed industrial historian into a closed-loop decision system with explainable incidents, operator actions, and Merkle-DAG-backed replay.

*Screenshot: local dev server — `cargo run -p vigil-cli -- daemon`. Alternate view: [`docs/landing-page-alt.png`](docs/landing-page-alt.png).*

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/AngelP17/ForgeMesh/actions/workflows/ci.yml/badge.svg)](https://github.com/AngelP17/ForgeMesh/actions)

*CI badge targets the default remote for this project (`AngelP17/ForgeMesh`). If you fork, point the badge at your repo.*

## Problem

Shift supervisors lose time hunting across siloed machine logs, maintenance records, and operator notes. Raw anomalies are not enough. Teams need explainable incidents, recommended actions, and trustworthy replay of why a system made a decision.

## Solution

Vigil ingests noisy multi-source manufacturing data, detects explainable incidents, recommends next actions, records operator decisions, and preserves Merkle-DAG-backed replay for integrity and trust.

**Zero recurring cost (product goal):** the default path is self-hosted on your hardware—no required SaaS, cloud database, LLM API, or subscription. Optional hooks (e.g. Slack incoming webhooks) are add-ons and never required for incidents, detection, replay, or the dashboard.

## Workflow

`Ingest → Detect → Explain → Recommend → Act → Replay`

## Why This Matters in High-Stakes Operations

- local-first operation for degraded or partitioned industrial networks, with **$0 recurring vendor cost** on the core hot path (see Solution above)
- human-in-the-loop action handling instead of black-box anomaly dashboards
- replayable incident reasoning with Merkle-backed integrity verification
- production-minded persistence split: Sled for telemetry, SQLite for operational workflow state

## Key Features

- Three v1 incident patterns: `temp_spike`, `vibration_anomaly`, `multi_machine_cascade`
- Three data sources: machine logs, maintenance tickets, operator notes
- **Five** operator actions (not three): acknowledge, assign maintenance, reroute, override, resolve
- Incidents list, incident detail, and replay/audit surfaces in the Axum dashboard; **Sensor trends** (Chart.js); **Mesh topology** from the live `vigil-p2p` gossip engine (`GET /api/mesh/topology`); **sensor CAR download** (`GET /api/export/:id/car`); PDF export for incidents (`/api/incidents/:id/export/pdf`)
- Read-first incident copilot for summary, explanation, handoff, and bounded Q&A
- Local demo seeding with nulls, duplicates, delays, out-of-order events, and conflicting notes

## Architecture

```mermaid
flowchart LR
    PLC[Machine PLC Logs] --> Detect[Vigil Incident Loop]
    Tickets[Maintenance Tickets] --> Detect
    Notes[Operator Notes] --> Detect
    Detect --> SQLite[(SQLite Incident Store)]
    Detect --> Replay[Merkle Replay + Proof]
    Detect --> UI[Axum Dashboard + WebSocket]
    Sled[(Sled Merkle DAG)] --> Detect
```

Telemetry remains in the ForgeMesh substrate:

- Sled-backed immutable telemetry chains
- Merkle verification for tamper evidence
- Axum + WebSocket dashboard
- local-first CLI and demo workflow

Vigil adds:

- incident persistence and status transitions
- operator action recording
- decision audit log with reasoning snapshots
- health endpoint and incident/replay APIs

## Quick Start

```bash
cargo build

# Seed noisy data and create incidents
cargo run -p vigil-cli -- seed-demo
cargo run -p vigil-cli -- detect

# Launch Vigil
cargo run -p vigil-cli -- daemon --port 8080
```

Open `http://localhost:8080`.

Useful commands:

```bash
# Verify telemetry chain integrity
cargo run -p vigil-cli -- verify -s ontario-line1-temp

# Export sample data again
./scripts/seed_demo_data.sh

# Run the full local demo flow
./scripts/run_demo_flow.sh
```

## API

```text
GET  /api/incidents
GET  /api/incidents?severity=&status=&machine=&q=&from=&to=&tenant_id=
GET  /api/incidents/export/csv
GET  /api/incidents/:id
GET  /api/incidents/:id/export/json
GET  /api/incidents/:id/export/pdf
GET  /api/incidents/:id/report
GET  /api/incidents/:id/notify/mailto
GET  /api/mesh/topology
POST /api/export/:id
GET  /api/export/:id/car
GET  /api/integrations/slack
PUT  /api/integrations/slack
POST /api/integrations/slack/test
POST /api/incidents/:id/copilot
GET  /api/incidents/:id/replay
POST /api/incidents/:id/actions
POST /api/auth/login
POST /api/auth/logout
GET  /api/auth/me
GET  /api/health
GET  /api/status
GET  /api/copilot/status
POST /api/detection/run
GET  /api/sensors
GET  /api/sensor/:id/history
GET  /api/sensor/:id/analytics
```

Environment (optional):

- `VIGIL_REQUIRE_AUTH=true` — require `Authorization: Bearer <token>` on write/simulation/detection/actions/copilot/reorder
- `VIGIL_ENFORCE_TENANT_SCOPE=true` — when set, signed-in operators with role `operator` (not `supervisor` / `admin`) only see incidents matching their `tenant_id` (CSV/list/detail/replay/exports respect the same rule)
- `VIGIL_SLACK_WEBHOOK_URL` — Slack incoming webhook for **critical** incidents after detection (optional; dashboard **System Health** can also persist a URL in SQLite via `PUT /api/integrations/slack`, `admin`/`supervisor` only — env wins on restart if set)

Default operator (first database init): username `operator`, password `vigil`. Create more with  
`cargo run -p vigil-cli -- create-user --username alice --password '...' --role supervisor`.

## Integrity and Replay

Each incident stores:

- timeline snapshot
- rule fired
- reasoning text
- Merkle root
- operator action history

Replay responses include the verification string (exact characters returned by the API; see `crates/vigil-core/src/audit.rs`):

```text
Valid Merkle path - data untampered
```

Use ASCII hyphen-minus (`U+002D`) between `path` and `data`, not an en dash or em dash.

Copilot responses are also written into replay as read-only audit entries.

## Read-First Copilot

The copilot is intentionally narrow:

- summarizes the incident
- explains why the incident fired
- prepares a shift handoff note
- answers bounded read-only questions grounded in incident, replay, health, and telemetry context

It does not execute actions or change state. The implementation details and 30-day rollout are documented in [docs/vigil-agent.md](docs/vigil-agent.md) (paths in this README are repo-relative to the workspace root).

## Demo Assets

- script: [demo/demo_script.md](demo/demo_script.md)
- scenario: [demo/demo_scenario.md](demo/demo_scenario.md)
- screenshots directory: [demo/screenshots/README.md](demo/screenshots/README.md)
- regenerate PNGs (requires a running daemon and [Playwright](https://playwright.dev/)):  
  `cd demo/screenshots && npm install && npx playwright install chromium && node capture.mjs`  
  (with `cargo run -p vigil-cli -- daemon --port 8080` in another terminal)

### Screenshots

![Incident list and dashboard](demo/screenshots/incident-list.png)
![Incident detail](demo/screenshots/incident-detail.png)
![Replay and operator workflow](demo/screenshots/replay-view.png)
![Health card](demo/screenshots/health-card.png)
![Sensor trends (Chart.js)](demo/screenshots/sensor-trends.png)

## Why Vigil Fits High-Stakes Ops Roles

- end-to-end ownership of an operational incident workflow
- explainable decision support under messy, conflicting data
- human-in-the-loop actions with status transitions and audit history
- cryptographic auditability and replay surfaced directly in product UX
- modular Rust backend design with clear storage, API, and workflow boundaries
