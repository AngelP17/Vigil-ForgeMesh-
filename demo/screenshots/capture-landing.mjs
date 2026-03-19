/**
 * Capture only the Vigil marketing page in Chromium (no IDE/desktop) → docs/readme-vigil-landing.png
 * Requires a running daemon, e.g.:
 *   cargo run -p vigil-cli -- daemon --port 8080
 *
 *   cd demo/screenshots && npm install && npx playwright install chromium
 *   node capture-landing.mjs
 */
import { chromium } from 'playwright';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const out = join(__dirname, '../../docs/readme-vigil-landing.png');
const base = process.env.VIGIL_BASE_URL || 'http://127.0.0.1:8080';

async function main() {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
    deviceScaleFactor: 1,
  });
  await page.goto(`${base}/`, { waitUntil: 'domcontentloaded', timeout: 90000 });
  await page.waitForSelector('nav', { timeout: 30000 });
  await page.waitForTimeout(2000);
  await page.screenshot({
    path: out,
    fullPage: false,
    type: 'png',
  });
  await browser.close();
  console.log('Wrote', out);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
