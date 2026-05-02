/**
 * Capture dashboard PNGs for README with new React frontend.
 * Requires a running Vigil daemon, e.g.:
 *   cargo run -p vigil-cli -- daemon --port 8080
 */
import { chromium } from 'playwright';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const base = process.env.VIGIL_BASE_URL || 'http://127.0.0.1:8080';

async function shot(page, name) {
  await page.screenshot({
    path: join(__dirname, name),
    fullPage: false,
  });
}

async function main() {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
  });

  // Dashboard - Incidents
  await page.goto(`${base}/dashboard#incidents`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(2000);
  await shot(page, 'incident-list.png');

  // Health
  await page.goto(`${base}/dashboard#health`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(1500);
  await shot(page, 'health-card.png');

  // Telemetry
  await page.goto(`${base}/dashboard#telemetry`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(2000);
  await shot(page, 'sensor-trends.png');

  // Incident Detail
  await page.goto(`${base}/dashboard#incidents`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(1000);
  const cards = await page.locator('.incident-card').count();
  if (cards > 0) {
    await page.locator('.incident-card').first().click();
    await page.waitForTimeout(2000);
    await shot(page, 'incident-detail.png');
  } else {
    await shot(page, 'incident-detail.png');
  }

  await browser.close();
  console.log('Screenshots written to', __dirname);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
