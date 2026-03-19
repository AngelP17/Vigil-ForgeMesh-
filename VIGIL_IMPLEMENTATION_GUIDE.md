# Vigil — End-to-End Implementation Guide

## Purpose

This document is the **single build spec** for evolving **ForgeMesh** into **Vigil**, a closed-loop operational incident intelligence platform.

It is designed to:
- preserve what already exists and is strong in ForgeMesh
- add only the missing operational workflow pieces
- stay zero-cost, local-first, and recruiter-readable
- map cleanly to Palantir-style signals across both:
  - **Forward Deployed Enablement Engineer**
  - **Software Engineer, New Grad**

This guide assumes:
- ForgeMesh is already running
- Merkle-DAG, simulation, analytics, Axum dashboard, WebSocket updates, and existing storage primitives already exist
- the goal is **not** to build a new app
- the goal is to transform ForgeMesh into **Vigil**

---

# 1. Product Positioning

## Final name

**Vigil**

Why this name works:
- one word
- modern and brandable
- operational and high-stakes in tone
- fits industrial intelligence and incident workflows
- reads like a real product, not a school project

## Product definition

**Vigil** is an operational incident intelligence platform that turns noisy multi-source manufacturing data into explainable incidents, recommended actions, operator decisions, and replayable audit trails.

## Core workflow

```text
Ingest → Detect → Explain → Recommend → Act → Replay
```

That workflow is the center of the project. Everything that does not support that loop is secondary.

---

# 2. What already exists and should be preserved

ForgeMesh already has real strengths. Do not bury them under unnecessary rewrites.

## Existing strengths to preserve

- **Merkle-DAG auditability**
  - strongest trust and traceability signal in the project
  - should remain the backbone of replay and integrity verification

- **simulation.rs**
  - already gives you synthetic operational data generation
  - should be extended, not replaced

- **analytics.rs**
  - already gives you anomaly and edge analytics foundations
  - should feed the incident engine

- **Axum dashboard + WebSocket** (static HTML/CSS/JS; the landing page uses inline SVG/CSS for illustrative “integrity” graphics; **only** the dashboard **Sensor trends** view loads **Chart.js** from a CDN for `/api/sensor/:id/history` — not used for the landing page charts)
  - already gives you a live operational UI surface
  - should be extended with incidents, details, and replay

- **Sled / existing storage primitives**
  - preserve where useful
  - only add SQLite where relational operational objects make the system much cleaner

- **multi-site / mesh context**
  - this is a real differentiator for manufacturing and Palantir-style deployment narratives

## Architectural principle

Do **not** replace ForgeMesh.
Do **not** restart from scratch.
Do **not** convert this into a generic monitoring dashboard.

The right transformation is:

> ForgeMesh remains the industrial data substrate. Vigil becomes the operational decision layer on top.

---

# 3. Palantir signal mapping

This project should make a reviewer infer the following immediately.

## Forward Deployed Enablement Engineer signals

Vigil should show:
- incident triage under ambiguity
- helping operators act on system recommendations
- clear explanation of why the system made a decision
- human-in-the-loop execution
- first-responder workflow ownership
- debugging with incomplete or conflicting data

## Software Engineer, New Grad signals

Vigil should show:
- real backend feature design
- API design and state transitions
- data modeling for operational systems
- modular architecture
- clean persistence layer
- production-minded engineering with CI, health surfaces, and traceability

## Shared overlap to optimize for

The overlap is the strongest target:

> a real operational system with strong engineering underneath

That is what Vigil should become.

---

# 4. Repo-level changes to make immediately

These are high ROI and low effort.

## 4.1 Rename repository

Rename:

```text
ForgeMesh → vigil
```

Global replace:
- Cargo.toml workspace names
- all crate Cargo.toml files
- Justfile
- ARCHITECTURE.md
- EXTENSIONS.md
- module paths and references

Suggested commit:

```text
Rename ForgeMesh to Vigil: operational incident intelligence platform
```

## 4.2 Update repo description

Suggested repo description:

```text
Vigil: Operational Incident Intelligence Platform. Evolved from ForgeMesh distributed industrial historian into a closed-loop decision system with full Merkle-DAG auditability.
```

## 4.3 Pin repo on profile

Pin Vigil in your top repositories.

## 4.4 Add GitHub topics

Suggested topics:
- rust
- industrial-iot
- time-series
- merkle-dag
- operational-intelligence
- incident-management
- manufacturing
- axum
- edge-analytics

## 4.5 Add CI workflow

Create:

```text
.github/workflows/ci.yml
```

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --all-features
      - run: cargo test
```

README badge:

```md
[![CI](https://github.com/AngelP17/ForgeMesh/actions/workflows/ci.yml/badge.svg)](https://github.com/AngelP17/ForgeMesh/actions)
```

This is a small addition, but it materially improves production credibility.

---

# 5. Scope freeze

This is the final v1 scope. Do not exceed it until it is fully shipped.

## 5.1 Data sources

Exactly 3:
- machine logs
- maintenance tickets
- operator notes

## 5.2 Incident patterns

Exactly 3:
- temperature spike
- vibration anomaly
- multi-machine cascade

## 5.3 UI surfaces

Exactly 3:
- incidents list
- incident detail
- replay / audit view

## 5.4 Operator actions

Exactly these actions:
- acknowledge
- assign_maintenance
- reroute
- override
- resolve

## 5.5 Non-goals for v1

Do not build:
- auth
- cloud deployment
- rule editor
- ML-heavy models
- notifications
- role system
- dynamic ontology editor
- major frontend redesign
- broad microservice split
- new distributed infrastructure layers unless already trivial from existing ForgeMesh

---

# 6. Recommended architecture

## 6.1 Architecture principle

Keep Vigil **Rust-first** and **local-first**.

Use the current ForgeMesh stack for:
- ingestion and simulation
- analytics
- Merkle-DAG lineage
- Axum API and UI
- WebSocket refreshes

Add a lightweight relational layer using SQLite only for the operational objects that benefit from clear stateful querying.

## 6.2 Why SQLite is correct here

SQLite is the correct choice for v1 because it is:
- zero cost
- embedded
- local-first
- easy to ship in demos
- strong enough for incident lifecycle storage

Use SQLx with SQLite for:
- incidents
- decision audit log
- operator actions
- health metrics if needed

Keep using existing ForgeMesh structures where they already make sense.

---

# 7. Exact folder structure

Recommended structure after transformation:

```text
vigil/
├── README.md
├── ARCHITECTURE.md
├── EXTENSIONS.md
├── demo/
│   ├── demo_script.md
│   ├── demo_scenario.md
│   ├── demo_video_link.txt
│   └── screenshots/
├── .github/
│   └── workflows/
│       └── ci.yml
├── migrations/
│   └── 001_incident_intelligence.sql
├── data/
│   ├── sample_machine_logs.jsonl
│   ├── sample_maintenance_tickets.csv
│   └── sample_operator_notes.jsonl
├── crates/
│   ├── forgemesh-core/
│   │   └── src/
│   │       ├── db.rs
│   │       ├── models.rs
│   │       ├── incidents.rs
│   │       ├── rules.rs
│   │       ├── audit.rs
│   │       ├── actions.rs
│   │       ├── analytics.rs
│   │       ├── simulation.rs
│   │       ├── merkle.rs
│   │       └── ...
│   ├── forgemesh-web/
│   │   └── src/
│   │       ├── api.rs
│   │       ├── incidents_ui.rs
│   │       ├── replay_ui.rs
│   │       ├── health_ui.rs
│   │       └── main.rs
├── scripts/
│   ├── seed_demo_data.sh
│   └── run_demo_flow.sh
├── Dockerfile
├── docker-compose.yml
└── tests/
    ├── test_rules.rs
    ├── test_incidents.rs
    ├── test_replay.rs
    └── test_actions.rs
```

Notes:
- keep names if you do not want to rename crate paths immediately
- repo can be renamed to Vigil even if crate names remain transitional for a short time
- this structure assumes feature-based additions without deleting existing working code

---

# 8. Dependencies

## 8.1 Add to core crate

File:

```text
crates/forgemesh-core/Cargo.toml
```

Add:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
```

Then run:

```bash
cargo check
```

---

# 9. Database layer

## 9.1 Migration file

Create:

```text
migrations/001_incident_intelligence.sql
```

Use this exact schema for v1.

```sql
CREATE TABLE IF NOT EXISTS machines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    location TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS raw_events (
    id TEXT PRIMARY KEY,
    machine_id TEXT,
    source TEXT NOT NULL,
    raw_timestamp TEXT,
    ingested_at TEXT,
    payload_json TEXT,
    is_valid INTEGER DEFAULT 1,
    validation_notes TEXT
);

CREATE TABLE IF NOT EXISTS maintenance_tickets (
    id TEXT PRIMARY KEY,
    machine_id TEXT NOT NULL,
    opened_at TEXT NOT NULL,
    closed_at TEXT,
    ticket_type TEXT,
    status TEXT NOT NULL,
    description TEXT
);

CREATE TABLE IF NOT EXISTS incidents (
    id TEXT PRIMARY KEY,
    machine_id TEXT,
    incident_type TEXT,
    severity TEXT,
    status TEXT DEFAULT 'open',
    title TEXT,
    suspected_cause TEXT,
    recommended_action TEXT,
    opened_at TEXT,
    closed_at TEXT
);

CREATE TABLE IF NOT EXISTS decision_audit_log (
    id TEXT PRIMARY KEY,
    incident_id TEXT,
    stage TEXT,
    rule_id TEXT,
    rule_version TEXT,
    inputs_snapshot_json TEXT,
    reasoning_text TEXT,
    merkle_root TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS operator_actions (
    id TEXT PRIMARY KEY,
    incident_id TEXT,
    action_type TEXT,
    action_note TEXT,
    taken_by TEXT,
    taken_at TEXT
);

CREATE TABLE IF NOT EXISTS pipeline_runs (
    id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL,
    events_processed INTEGER DEFAULT 0,
    incidents_created INTEGER DEFAULT 0,
    invalid_events INTEGER DEFAULT 0,
    error_summary TEXT
);
```

## 9.2 DB bootstrap file

Create:

```text
crates/forgemesh-core/src/db.rs
```

Responsibilities:
- initialize SQLite DB file
- run migration on startup
- expose shared pool type

Suggested implementation responsibilities:
- build `sqlite://` URL
- create DB if missing
- connect with SQLx pool
- execute migration file
- return shared pool for Axum extension injection

---

# 10. Core module responsibilities

## 10.1 models.rs

Create:

```text
crates/forgemesh-core/src/models.rs
```

This file defines the core operational structs.

Required models:
- `Incident`
- `DecisionAuditLog`
- `OperatorAction`

Recommended fields:
- match schema 1:1 where practical
- derive `Debug`, `Serialize`, `Deserialize`, `sqlx::FromRow`

Suggested constructor helpers:
- `Incident::new(...)`
- `OperatorAction::new(...)`

This keeps rule code cleaner and reduces duplication.

---

## 10.2 incidents.rs

Create:

```text
crates/forgemesh-core/src/incidents.rs
```

Responsibilities:
- create incident rows
- list incidents
- fetch single incident
- update status
- optionally attach recommendation updates later

Required functions:

```rust
pub async fn create_incident(pool: &SqlitePool, incident: Incident) -> sqlx::Result<String>;
pub async fn list_incidents(pool: &SqlitePool) -> sqlx::Result<Vec<Incident>>;
pub async fn get_incident(pool: &SqlitePool, id: &str) -> sqlx::Result<Option<Incident>>;
pub async fn update_status(pool: &SqlitePool, id: &str, status: &str) -> sqlx::Result<()>;
```

Behavior notes:
- `create_incident()` should ensure UUID exists
- `update_status()` should set `closed_at` when status becomes resolved
- keep logic simple and deterministic

---

## 10.3 rules.rs

Create:

```text
crates/forgemesh-core/src/rules.rs
```

Responsibilities:
- convert anomalous or correlated event groups into operational incidents
- assign initial severity
- provide recommendation and reasoning text input for audit logging

Only these 3 rules belong in v1.

### Rule 1: Temp spike

Trigger when:
- temperature exceeds threshold
- or z-score exceeds threshold
- optionally increase severity if recent maintenance ticket exists for same machine

Suggested outcome:
- `incident_type = temp_spike`
- recommendation: reduce load, inspect cooling path, dispatch mechanic

### Rule 2: Vibration anomaly

Trigger when:
- vibration crosses threshold
- or sudden delta compared to recent history

Suggested outcome:
- `incident_type = vibration_anomaly`
- recommendation: assign maintenance inspection, inspect bearings or rotating assembly

### Rule 3: Multi-machine cascade

Trigger when:
- 3 or more machines emit critical or failing events inside a short time window
- best used when mesh or multi-site data makes correlated failure visible

Suggested outcome:
- `incident_type = multi_machine_cascade`
- severity should be highest
- recommendation: investigate shared infrastructure and reroute work

Required function:

```rust
pub fn detect_incidents(events: &[crate::types::DataNode]) -> Vec<Incident>;
```

Recommended design:
- keep rules composable
- allow chained severity increases
- do not overabstract

---

## 10.4 audit.rs

Create:

```text
crates/forgemesh-core/src/audit.rs
```

Responsibilities:
- write decision audit records
- compute and store Merkle root
- reconstruct replay payload
- optionally return proof path for integrity verification

This file is the strongest trust feature in the entire system.

Required function:

```rust
pub async fn log_decision(
    pool: &SqlitePool,
    incident_id: &str,
    snapshot: serde_json::Value,
    rule_id: &str,
    reasoning: &str,
) -> sqlx::Result<()>;
```

Required behavior:
- compute Merkle root from snapshot or event lineage payload
- store snapshot JSON
- store reasoning text
- store stage and rule version

Required replay function:

```rust
pub async fn get_replay(pool: &SqlitePool, incident_id: &str) -> sqlx::Result<serde_json::Value>;
```

Replay response should include:

```json
{
  "incident_id": "...",
  "timeline": [...],
  "rules_fired": [...],
  "reasoning": "...",
  "merkle_root": "...",
  "proof": [...],
  "operator_actions": [...],
  "verification": "Valid Merkle path - data untampered"
}
```

### Integrity verification enhancement

High-ROI addition:
- reuse existing `merkle.rs` verification function
- return proof path in replay payload
- add a UI action for `Verify Integrity`

This is an unusually strong signal and worth surfacing visibly.

---

## 10.5 actions.rs

Create:

```text
crates/forgemesh-core/src/actions.rs
```

Responsibilities:
- record operator actions
- update incident status
- optionally broadcast dashboard refresh through existing WebSocket path

Required function:

```rust
pub async fn take_action(
    pool: &SqlitePool,
    incident_id: &str,
    action_type: &str,
    note: &str,
    taken_by: &str,
) -> sqlx::Result<()>;
```

Expected behavior:
- insert row into `operator_actions`
- update incident status
- set `closed_at` if action resolves incident
- if WebSocket refresh is already wired in project, trigger refresh after mutation

Suggested action-to-status mapping:
- `acknowledge` → `acknowledged`
- `assign_maintenance` → `assigned`
- `reroute` → `rerouted`
- `override` → remains open or acknowledged depending on your workflow
- `resolve` → `resolved`

---

# 11. Simulation realism upgrades

File to extend:

```text
crates/forgemesh-core/src/simulation.rs
```

This is a high-signal area. It is worth making the messiness explicit.

## 11.1 Required source types

Add 3 simulated sources:
- `machine_plc`
- `maintenance_ticket`
- `operator_note`

## 11.2 Required data messiness

Inject:
- null values
- duplicate events
- delayed arrival
- out-of-order timestamps
- conflicting observations
- schema variance if feasible

## 11.3 Example realism pattern

Examples of ambiguity to simulate:
- PLC reports temperature 90
- operator note says cooling was manually applied and temp appears lower
- maintenance ticket references prior fan issue on same line

That creates a believable operator context instead of a toy anomaly demo.

## 11.4 Suggested metadata pattern

Store source realism metadata in event payload or metadata fields, for example:
- source type
- noise probabilities
- delay seconds
- duplicate marker
- confidence score if useful

## 11.5 Demo data export

Write sample JSONL / CSV to:

```text
data/
```

Suggested files:
- `sample_machine_logs.jsonl`
- `sample_maintenance_tickets.csv`
- `sample_operator_notes.jsonl`

These become valuable for the demo, README, and reproducible local runs.

---

# 12. Analytics and detection wiring

Files:
- `crates/forgemesh-core/src/analytics.rs`
- `crates/forgemesh-core/src/rules.rs`

## 12.1 Wiring goal

Do not let analytics end at anomaly detection.

The correct flow is:

```text
raw events → analytics candidate anomalies → rules → incidents → audit log
```

## 12.2 Integration pattern

After existing anomaly or OEE logic:
- derive candidate events or groups
- pass them into `rules::detect_incidents()`
- for each incident:
  1. insert incident
  2. build snapshot
  3. log decision

## 12.3 Snapshot contents

The decision snapshot should include enough context to explain the decision later.

Recommended contents:
- event IDs
- event timestamps
- machine IDs
- source types
- relevant metrics used in decision
- optional active ticket IDs
- optional anomaly score or z-score values

This turns replay into something useful rather than decorative.

---

# 13. API layer

File:

```text
crates/forgemesh-web/src/api.rs
```

Add routes:

```text
GET  /incidents
GET  /incidents/:id
GET  /incidents/:id/replay
POST /incidents/:id/actions
GET  /api/health
```

## 13.1 Incident list handler

Should return:
- incident id
- machine id
- type
- severity
- status
- recommendation
- opened_at

## 13.2 Incident detail handler

Should return:
- incident record
- recent related operator actions if convenient
- optionally linked maintenance tickets

## 13.3 Replay handler

Should return full replay JSON from `audit::get_replay()`.

## 13.4 Action handler

Should accept:

```json
{
  "action_type": "resolve",
  "note": "Mechanic dispatched and issue cleared",
  "taken_by": "Operator_1"
}
```

Then:
- write operator action
- update incident status
- broadcast update if WS refresh exists

## 13.5 Health endpoint

High ROI addition.

Suggested response:

```json
{
  "last_ingest": "...",
  "events_last_hour": 143,
  "incidents_open": 2,
  "invalid_events": 11,
  "mesh_nodes": 3,
  "data_quality": "92% valid"
}
```

This gives you operational credibility without much code.

---

# 14. UI surfaces

Use the existing Axum dashboard. Do not replace it.

## 14.1 Incidents list view

Create or extend:
- table or panel listing incidents

Show:
- incident ID
- machine
- type
- severity
- status
- recommendation
- opened time

This should be the first place a reviewer lands.

## 14.2 Incident detail view

Show:
- title
- suspected cause
- recommended action
- reasoning summary
- related maintenance reference if available
- action buttons
- operator note input

Buttons:
- acknowledge
- assign maintenance
- reroute
- override
- resolve

## 14.3 Replay view

This is the showcase surface.

Show:
- raw events in chronological order
- rules fired
- reasoning text
- Merkle root
- proof if available
- operator action history
- integrity verification result

## 14.4 Verify Integrity button

High signal and low effort.

Button action:
- call replay or verify endpoint
- reuse existing Merkle verification logic
- display success state such as:

```text
Tamper-evident: Valid
```

This is extremely strong for trust, lineage, and high-stakes systems narratives.

## 14.5 System health section

Add a small dashboard section or card showing:
- last ingest time
- invalid events
- incidents open
- data quality percentage
- mesh nodes count

This can be a compact card and still improve the product significantly.

---

# 15. README structure

The README is not optional polish. It is part of the signal.

## 15.1 Title

```md
# Vigil

**Operational Incident Intelligence Platform**

Evolved from ForgeMesh distributed industrial historian into a closed-loop decision system with explainable incidents, operator actions, and Merkle-DAG-backed replay.
```

## 15.2 Recommended sections

1. Problem
2. Solution
3. Workflow
4. Why this matters in high-stakes operations
5. Architecture
6. Key features
7. Integrity and replay
8. Demo
9. Quick start
10. Role relevance / why this maps to operational engineering

## 15.3 Problem section

Suggested framing:

> Shift supervisors lose time hunting across siloed machine logs, maintenance records, and operator notes. Raw anomalies are not enough. Teams need explainable incidents, recommended actions, and trustworthy replay of why a system made a decision.

## 15.4 Solution section

Suggested framing:

> Vigil turns noisy multi-source manufacturing data into a closed-loop operational workflow: it detects incidents, explains why they were created, recommends actions, records operator decisions, and preserves Merkle-DAG-backed audit trails for replay and integrity verification.

## 15.5 Add CI badge and screenshots

Include:
- CI badge
- incident list screenshot
- incident detail screenshot
- replay view screenshot
- health card screenshot if available

## 15.6 Add role relevance section

Suggested heading:

```md
## Why Vigil Fits High-Stakes Ops Roles
```

Bullets:
- end-to-end ownership of incident workflow
- explainable decision support under messy data
- human-in-the-loop actions
- cryptographic auditability and replay
- production-minded backend and API design

That section helps collapse ambiguity for recruiters.

---

# 16. Demo script

Create:

```text
demo/demo_script.md
```

## 16.1 Demo duration

Target 5 to 6 minutes.

## 16.2 Demo flow

1. Frame the problem
   - siloed logs across plants
   - operators need trusted actions, not just anomalies

2. Seed noisy data
   - show machine logs, tickets, and notes
   - briefly point out missing values, delays, and conflicts

3. Run detection
   - incident appears in dashboard

4. Open incident detail
   - show cause and recommendation
   - explain why the system flagged it

5. Take operator action
   - resolve, reroute, or assign maintenance with note

6. Open replay view
   - show timeline
   - show rule fired
   - show Merkle root or proof
   - press Verify Integrity if implemented

7. Close with product framing
   - this is a closed-loop operational decision system, not just a monitor

## 16.3 Demo sentence to use

Suggested line:

> Vigil mirrors Foundry-style operational workflows by converting noisy real-world signals into explainable incidents, operator actions, and replayable audit trails.

---

# 17. Wiring plan by file

This is the concrete implementation checklist.

## 17.1 Files to create

Create these files if missing:

```text
migrations/001_incident_intelligence.sql
crates/forgemesh-core/src/db.rs
crates/forgemesh-core/src/models.rs
crates/forgemesh-core/src/incidents.rs
crates/forgemesh-core/src/rules.rs
crates/forgemesh-core/src/audit.rs
crates/forgemesh-core/src/actions.rs
.github/workflows/ci.yml
demo/demo_script.md
```

## 17.2 Files to modify

Modify these existing files:

```text
crates/forgemesh-core/Cargo.toml
crates/forgemesh-core/src/simulation.rs
crates/forgemesh-core/src/analytics.rs
crates/forgemesh-web/src/api.rs
crates/forgemesh-web/src/main.rs
README.md
ARCHITECTURE.md
EXTENSIONS.md
Justfile
all crate Cargo.toml files that still reference ForgeMesh naming
```

## 17.3 Optional UI files to add if your current web layer benefits from separation

```text
crates/forgemesh-web/src/incidents_ui.rs
crates/forgemesh-web/src/replay_ui.rs
crates/forgemesh-web/src/health_ui.rs
```

---

# 18. Exact implementation order

Follow this order strictly.

## Phase 1 — repo and dependency setup

1. rename repo to Vigil
2. update repo description and topics
3. add CI workflow
4. add SQLx and other dependencies
5. run `cargo check`

**Done means:** repository identity is correct and build still compiles.

## Phase 2 — persistence foundation

1. add migration file
2. add `db.rs`
3. add `models.rs`
4. add `incidents.rs`
5. add `audit.rs`
6. add `actions.rs`

**Done means:** you can manually insert an incident, an audit row, and an operator action.

## Phase 3 — simulation realism

1. extend `simulation.rs`
2. add three source types
3. add noisy / conflicting behavior
4. export sample data files

**Done means:** a seeded run produces believable messy operational data.

## Phase 4 — analytics to incident flow

1. connect `analytics.rs` to `rules.rs`
2. create incidents from detections
3. create decision audit log rows
4. persist snapshots and Merkle root

**Done means:** a run auto-creates at least one incident and one audit record.

## Phase 5 — action loop

1. add action handler
2. update incident status transitions
3. record operator actions
4. wire WebSocket refresh if already available

**Done means:** operator action changes state and is visible in UI.

## Phase 6 — replay and integrity

1. implement `get_replay()`
2. expose replay endpoint
3. show replay in UI
4. add proof or verification status

**Done means:** replay returns events, reasoning, Merkle root, and actions.

## Phase 7 — health and polish

1. add `/api/health`
2. add health card in dashboard
3. add screenshots
4. record demo video
5. finalize README

**Done means:** system looks and reads like a real product.

---

# 19. Definition of shipped

Vigil is shipped when all of the following are true:

- repo renamed and polished
- CI badge green
- migration applies cleanly
- simulated noisy data populates the system
- at least one incident auto-creates from that data
- audit row includes reasoning and Merkle root
- operator action records and updates status
- replay endpoint returns full JSON with timeline and actions
- dashboard shows incidents list, detail, and replay
- README includes problem, solution, workflow, screenshots, and demo

If any of those are missing, it is not fully shipped.

---

# 20. Why this is the correct final form

This design avoids the two biggest mistakes:

## Mistake 1: overbuilding

Avoided by:
- freezing rules to three
- freezing UI surfaces to three
- not rewriting the core stack
- not adding cloud, auth, or ML sprawl

## Mistake 2: underselling existing strengths

Avoided by:
- centering Merkle-DAG replay
- preserving Axum dashboard
- preserving analytics
- preserving simulation
- preserving multi-site mesh context

That combination is what upgrades ForgeMesh from an industrial historian into a true operational intelligence system.

---

# 21. Final positioning statement

Use this wording internally and externally:

> Vigil is an operational incident intelligence platform evolved from ForgeMesh. It ingests noisy multi-source manufacturing data, detects explainable incidents, recommends next actions, records operator decisions, and preserves Merkle-DAG-backed replay for integrity and trust.

This is the positioning that best maps to both target Palantir role families.

---

# 22. Immediate next actions

Do these next, in order:

1. rename repo to Vigil
2. add `ci.yml`
3. add `001_incident_intelligence.sql`
4. add `db.rs`
5. add `models.rs`
6. add `incidents.rs`
7. add `audit.rs`
8. add `actions.rs`
9. extend `simulation.rs`
10. wire `analytics.rs` into `rules.rs`
11. expose incident and replay routes in `api.rs`
12. add replay and health surfaces to UI
13. record demo
14. lock README

At that point, the project stops being a promising concept and becomes a flagship systems artifact.

---

# 23. Short version to remember

If you need one line to stay disciplined, use this:

> Preserve ForgeMesh, add the incident loop, surface the audit trail, ship the operator workflow.

