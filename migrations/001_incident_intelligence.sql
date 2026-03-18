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
