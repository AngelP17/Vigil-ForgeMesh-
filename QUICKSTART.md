# Vigil Quick Start Guide

## Prerequisites

- Rust 1.75+ installed
- SQLite installed (for database)

## 1. Build the Project

```bash
cargo build --release
```

## 2. Initialize the Database

The database will be automatically created on first run, but you can seed it with demo data:

```bash
# Seed demo data (machine logs, maintenance tickets, operator notes)
cargo run -p vigil-cli -- seed-demo

# Run incident detection
cargo run -p vigil-cli -- detect
```

## 3. Start the Server

```bash
cargo run -p vigil-cli -- daemon --port 8080
```

Or use the release build for better performance:

```bash
./target/release/vigil-cli daemon --port 8080
```

## 4. Access the Dashboard

Open your browser and navigate to:

- **Landing Page**: http://localhost:8080/
- **Dashboard**: http://localhost:8080/dashboard

## Features to Explore

### Landing Page (http://localhost:8080/)
- **Live Health Metrics**: Real-time system health pulled from `/api/health`
- **Feature Overview**: Six core capabilities explained
- **Workflow Visualization**: 5-step operational flow
- **Trust & Integrity**: Merkle-backed audit visualization
- **Click "Open Dashboard"** to access the full interface

### Dashboard (http://localhost:8080/dashboard)

#### 1. **Incident List View**
   - Real-time incident cards with severity badges
   - Filter by status (open, acknowledged, assigned, resolved)
   - Live incident count in sidebar
   - Click any incident card to view details

#### 2. **Incident Detail View**
   - Complete incident information
   - Suspected cause and recommended action
   - Action timeline showing all operator interventions
   - **Operator Actions Panel**:
     - Acknowledge incident
     - Assign maintenance
     - Reroute operations
     - Override recommendation
     - Resolve incident

#### 3. **Read-First Copilot**
   - **Summary**: Get a concise incident summary
   - **Explain**: Understand why the incident fired
   - **Handoff**: Generate shift handoff notes
   - **Q&A**: Ask specific questions about the incident
   - All copilot responses are audit-logged

#### 4. **Replay & Audit Trail**
   - Cryptographic Merkle verification
   - Complete event timeline
   - Rules fired
   - Reasoning text
   - Verification badge for tamper-evidence
   - Proof path visualization

#### 5. **WebSocket Live Updates**
   - Real-time incident notifications
   - Automatic health metric refreshes
   - Live event processing updates

## API Endpoints

### Health & Status
```bash
GET  /api/health
GET  /api/status
```

### Incidents
```bash
GET  /api/incidents                    # List all incidents
GET  /api/incidents/status/:status     # Filter by status
GET  /api/incidents/:id                # Get incident detail
POST /api/incidents/:id/actions        # Take operator action
GET  /api/incidents/:id/replay         # Get audit trail
```

### Copilot
```bash
POST /api/incidents/:id/copilot        # Run copilot
GET  /api/copilot/status               # Copilot availability
```

### Detection
```bash
POST /api/detection/run                # Run incident detection
```

### Sensors
```bash
GET  /api/sensors                      # List all sensors
GET  /api/sensor/:id/history           # Get sensor history
GET  /api/sensor/:id/analytics         # Get sensor analytics
POST /api/sensor/:id/simulate          # Simulate sensor data
```

## Testing the System

### 1. Seed Demo Data
```bash
cargo run -p vigil-cli -- seed-demo
```

This creates:
- Noisy machine telemetry with nulls, duplicates, and delays
- Maintenance tickets
- Conflicting operator notes

### 2. Run Detection
```bash
cargo run -p vigil-cli -- detect
```

This will:
- Analyze all data sources
- Detect incidents based on rules
- Create audit log entries
- Generate Merkle proofs

Or use the **"Run Detection"** button in the dashboard nav bar.

### 3. Interact with Incidents

In the dashboard:
1. View the incident list
2. Click on an incident to see details
3. Use the **Copilot** to get explanations:
   - Click "Summary" for a quick overview
   - Click "Explain" to understand the reasoning
4. Take an **Operator Action**:
   - Add a note (optional)
   - Click "Acknowledge" or "Assign Maintenance"
5. View the **Replay Trail**:
   - Click "View Replay & Audit Trail"
   - See Merkle verification
   - Review complete event timeline

### 4. Verify Integrity

The replay view shows cryptographic verification:
- Merkle root hash
- Proof path array
- Verification badge: "Valid Merkle path — data untampered"

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Vigil Dashboard                       │
│         (Landing Page + Real-Time Dashboard)            │
└─────────────────────┬───────────────────────────────────┘
                      │ WebSocket + REST API
┌─────────────────────┴───────────────────────────────────┐
│                   Axum Web Server                        │
│            (vigil-web with WebSocket support)            │
└─────────┬───────────────────────────┬───────────────────┘
          │                           │
┌─────────┴────────┐       ┌──────────┴──────────┐
│  SQLite Database │       │   Sled Merkle-DAG   │
│  - Incidents     │       │  - Telemetry Chains │
│  - Audit Logs    │       │  - Immutable Data   │
│  - Actions       │       │  - Verification     │
└──────────────────┘       └─────────────────────┘
```

### Data Flow

```
Machine Logs ──┐
Tickets       ─┼──> Ingest ──> Detect ──> Incidents ──> Dashboard
Operator Notes─┘                 │                          │
                                 ├──> Audit Log            │
                                 └──> Merkle Proof         │
                                                           │
                                                           └──> Operator Actions ──> Update
```

## Workflow Example

### Scenario: Temperature Spike Detected

1. **Detection**:
   - System ingests temperature data: 87°C (threshold: 85°C)
   - Correlates with maintenance ticket from 2 days ago
   - Creates incident: `temp_spike`
   - Logs decision with Merkle root

2. **Notification**:
   - WebSocket pushes incident to dashboard
   - Incident appears in list with "Critical" severity
   - Live count updates in sidebar

3. **Operator Response**:
   - Operator clicks incident card
   - Reads suspected cause: "Temperature exceeds threshold, recent maintenance ticket indicates cooling system issue"
   - Uses copilot "Explain" mode to understand
   - Adds note: "Dispatching mechanic to inspect cooling system"
   - Clicks "Assign Maintenance"
   - Status updates to "Assigned"

4. **Audit Trail**:
   - All actions recorded with timestamps
   - Copilot responses logged
   - Merkle proof generated
   - Replay view shows complete decision chain

5. **Resolution**:
   - Mechanic fixes cooling system
   - Operator adds note: "Cooling fan replaced, temp normalized"
   - Clicks "Resolve"
   - Incident closed with full audit trail

## Customization

### Add New Incident Rules

Edit `crates/vigil-core/src/rules.rs` to add custom detection rules:

```rust
// Example: Pressure anomaly rule
if sensor_id.contains("pressure") && value > threshold {
    incidents.push(Incident::new(
        machine_id,
        "pressure_anomaly",
        "critical",
        "Pressure spike detected",
        "Pressure exceeds safe operating threshold",
        "Reduce pressure, inspect relief valve"
    ));
}
```

### Customize Copilot Modes

Edit `crates/vigil-core/src/copilot.rs` to modify copilot behavior.

### Configure Health Metrics

Edit `crates/vigil-core/src/db.rs` to customize health snapshot calculations.

## Troubleshooting

### "No incidents detected"
- Run `cargo run -p vigil-cli -- seed-demo` to generate demo data
- Run `cargo run -p vigil-cli -- detect` to trigger detection
- Or use the "Run Detection" button in the dashboard

### WebSocket not connecting
- Check that the server is running on the correct port
- Ensure no firewall is blocking WebSocket connections
- Check browser console for errors

### Database errors
- Delete `vigil.db` and restart to reset database
- Check SQLite is properly installed
- Ensure write permissions in the working directory

### No copilot responses
- Copilot runs rule-based logic, not LLM calls
- Check `crates/vigil-core/src/copilot.rs` for implementation
- Ensure incident detail loaded correctly

## Next Steps

1. **Customize Incident Rules**: Add domain-specific detection logic
2. **Deploy Multi-Node**: Use `vigil-p2p` for distributed deployment
3. **Integrate Real Data**: Replace simulation with actual PLC/SCADA feeds
4. **Add Authentication**: Implement operator identity management
5. **Export Audit Trails**: Build CAR (Content Addressable Archive) export

## Documentation

- [README.md](README.md) - Main project documentation
- [VIGIL_IMPLEMENTATION_GUIDE.md](VIGIL_IMPLEMENTATION_GUIDE.md) - Implementation details
- [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
- [demo/demo_script.md](demo/demo_script.md) - Demo walkthrough

## Support

For questions or issues:
1. Check existing documentation
2. Review demo scenario: `demo/demo_scenario.md`
3. Open an issue on GitHub

---

**Vigil** — Operational Incident Intelligence
*Local-first · Replay-native · Built for manufacturing*
