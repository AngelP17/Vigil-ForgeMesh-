/**
 * Capture the Vigil marketing landing (/) for README docs/landing-page.png.
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
const out = join(__dirname, '../../docs/landing-page.png');
const base = process.env.VIGIL_BASE_URL || 'http://127.0.0.1:8080';

async function main() {
  const browser = await chromium.launch();
  const page = await browser.newPage({
    viewport: { width: 1440, height: 900 },
  });
  await page.goto(`${base}/`, { waitUntil: 'domcontentloaded', timeout: 60000 });
  await page.waitForTimeout(2500);
  await page.screenshot({
    path: out,
    fullPage: false,
  });
  await browser.close();
  console.log('Wrote', out);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
