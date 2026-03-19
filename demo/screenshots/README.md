# Dashboard screenshots (README)

PNG files referenced from the root [README](../../README.md):

| File | View |
|------|------|
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

Optional: set `VIGIL_BASE_URL` if the server uses another host/port.
