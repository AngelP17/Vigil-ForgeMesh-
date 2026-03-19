/**
 * Capture dashboard PNGs for README. Requires a running Vigil daemon, e.g.:
 *   cargo run -p vigil-cli -- daemon --port 8080
 *
 *   cd demo/screenshots && npm install && npx playwright install chromium
 *   VIGIL_BASE_URL=http://127.0.0.1:8080 node capture.mjs
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

  await page.goto(`${base}/dashboard`, {
    waitUntil: 'domcontentloaded',
    timeout: 60000,
  });
  await page.waitForTimeout(1200);
  await shot(page, 'incident-list.png');

  await page.getByTestId('nav-health').click();
  await page.waitForTimeout(600);
  await shot(page, 'health-card.png');

  await page.getByTestId('nav-telemetry').click();
  await page.waitForTimeout(1200);
  await shot(page, 'sensor-trends.png');

  await page.getByTestId('nav-incidents').click();
  await page.waitForTimeout(500);

  const card = page.locator('.incident-card').first();
  const n = await card.count();
  if (n > 0) {
    await card.click();
    await page.waitForTimeout(1000);
    await shot(page, 'incident-detail.png');
    await page.getByRole('button', { name: 'View Replay & Audit Trail' }).click();
    await page.waitForTimeout(900);
    await shot(page, 'replay-view.png');
  } else {
    console.warn(
      '[capture] No incidents — detail/replay shots duplicate list. Run daemon (seeds pipeline) or seed-demo + detect.',
    );
    await shot(page, 'incident-detail.png');
    await shot(page, 'replay-view.png');
  }

  await browser.close();
  console.log('Screenshots written to', __dirname);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
