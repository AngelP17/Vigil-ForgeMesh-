# Vigil Extension Notes

## Implemented (product)

### Operational workflow layer

- SQLite-backed incident storage with **tenant_id** and **SLA ack deadline** (`sla_ack_by`, default +4h from open)
- Operator actions with status transitions
- Decision audit log with reasoning snapshots
- Replay endpoint with Merkle verification output
- **Filtered incident listing** (`GET /api/incidents?severity=&machine=&q=&from=&to=&tenant_id=&status=`)
- **CSV export** (`GET /api/incidents/export/csv` with same query params)
- **JSON incident packet** (`GET /api/incidents/:id/export/json`)
- **Printable HTML report** (`GET /api/incidents/:id/report`)
- **PDF export** (`GET /api/incidents/:id/export/pdf`, built-in fonts; ASCII-safe text)
- **Mailto helper** for email drafts (`GET /api/incidents/:id/notify/mailto`)

### Auth (local-first)

- **Operators** table with bcrypt passwords, **sessions** with opaque tokens
- Default user on first DB init: `operator` / `vigil` (change in production)
- `POST /api/auth/login`, `POST /api/auth/logout`, `GET /api/auth/me`
- Optional enforcement: set **`VIGIL_REQUIRE_AUTH=true`** — then mutating APIs require `Authorization: Bearer <token>` (dashboard stores token after sign-in)
- Optional **tenant scope**: set **`VIGIL_ENFORCE_TENANT_SCOPE=true`** — authenticated `operator` accounts only see their `tenant_id`; roles `supervisor` and `admin` see all tenants
- CLI: `cargo run -p vigil-cli -- create-user --username ... --password ...`

### Notifications & integrations

- **Slack**: set **`VIGIL_SLACK_WEBHOOK_URL`** — critical incidents notify after detection runs
- **Browser**: dashboard requests notification permission; short tone on detection pipeline completion

### UI (dashboard)

- Filters, export CSV (authenticated when required), JSON download, printable report, mailto draft
- Dark/light **theme** toggle (persisted in `localStorage`)
- Sign in / sign out against `/api/auth/login`
- **Chart.js** sensor trend line chart on **Sensor trends** (data from `/api/sensor/:id/history`; refreshes on WebSocket activity while the view is open)
- **Mesh topology** view: JSON from **`GET /api/mesh/topology`** (illustrative snapshot; not live `vigil-p2p` sync)

### Intentionally not in scope

- Hosted multi-region “cloud deployment” as a product (local-first remains the default)
- Heavyweight ML / auto rule learning
- Third-party SSO (Okta, SAML) — only local operators DB

### Roadmap / deeper extensions

- Richer Merkle **D3** tree (current: list + proof strings + printable report + PDF summary)
- Site-to-site **p2p** sync wired to real gossip (`vigil-p2p` / `iroh` experimentation; dashboard shows static topology JSON today)
- Multi-tenant **isolation policies** beyond optional `VIGIL_ENFORCE_TENANT_SCOPE` + `tenant_id` filtering
