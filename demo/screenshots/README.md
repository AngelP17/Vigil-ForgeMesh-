# Dashboard screenshots (README)

PNG files referenced from the root [README](../../README.md):

| File | View |
|------|------|
| `../docs/landing-page.png` | Marketing landing `/` (see `capture-landing.mjs`) |
| `incident-list.png` | Incidents (default) |
| `incident-detail.png` | First incident detail |
| `replay-view.png` | Replay & audit after “View Replay & Audit Trail” |
| `health-card.png` | System Health |
| `sensor-trends.png` | Sensor trends (Chart.js) |

## Regenerate

1. Start the API + UI (daemon seeds demo data on startup):

   ```bash
   cargo run -p vigil-cli -- daemon --port 8080
   ```

2. In another terminal:

   ```bash
   cd demo/screenshots
   npm install
   npx playwright install chromium
   VIGIL_BASE_URL=http://127.0.0.1:8080 node capture.mjs
   ```

3. Landing page image for the root README (`docs/landing-page.png`):

   ```bash
   node capture-landing.mjs
   ```

Optional: set `VIGIL_BASE_URL` if the server uses another host/port.
