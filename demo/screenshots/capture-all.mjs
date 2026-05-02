/**
 * Capture ALL dashboard views with populated content.
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

  // Landing page
  await page.goto(`${base}/`, { waitUntil: 'networkidle', timeout: 60000 });
  await page.waitForTimeout(2500);
  await shot(page, '../../docs/readme-vigil-landing.png');

  // Dashboard - Incidents
  await page.goto(`${base}/dashboard#incidents`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(2500);
  await shot(page, 'incident-list.png');

  // Incident Detail - navigate directly via hash
  const firstId = await page.locator('.incident-card').first().getAttribute('data-id').catch(() => null);
  if (firstId) {
    await page.goto(`${base}/dashboard#detail/${firstId}`, {
      waitUntil: 'networkidle',
      timeout: 60000,
    });
    await page.waitForTimeout(2500);
    await shot(page, 'incident-detail.png');
  } else {
    // Fallback: try to get ID from text
    await page.goto(`${base}/dashboard#detail/37fc2928-3528-4893-ac88-8cd606d7bed6`, {
      waitUntil: 'networkidle',
      timeout: 60000,
    });
    await page.waitForTimeout(2500);
    await shot(page, 'incident-detail.png');
  }

  // Health
  await page.goto(`${base}/dashboard#health`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(2000);
  await shot(page, 'health-card.png');

  // Telemetry
  await page.goto(`${base}/dashboard#telemetry`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(2500);
  await shot(page, 'sensor-trends.png');

  // Mesh
  await page.goto(`${base}/dashboard#mesh`, {
    waitUntil: 'networkidle',
    timeout: 60000,
  });
  await page.waitForTimeout(1500);
  await shot(page, 'mesh-topology.png');

  await browser.close();
  console.log('All screenshots written to', __dirname);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
