CREATE TABLE IF NOT EXISTS tenants (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);

INSERT OR IGNORE INTO tenants (id, name) VALUES ('default', 'Default site');

CREATE TABLE IF NOT EXISTS operators (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'operator',
    tenant_id TEXT NOT NULL DEFAULT 'default',
    FOREIGN KEY (tenant_id) REFERENCES tenants(id)
);

CREATE INDEX IF NOT EXISTS idx_operators_tenant ON operators(tenant_id);

CREATE TABLE IF NOT EXISTS sessions (
    token TEXT PRIMARY KEY,
    operator_id TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (operator_id) REFERENCES operators(id)
);

CREATE INDEX IF NOT EXISTS idx_sessions_operator ON sessions(operator_id);
