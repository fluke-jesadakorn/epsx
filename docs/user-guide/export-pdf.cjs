// Run with NODE_PATH pointing to a Playwright installation.
const { chromium } = require('playwright');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
(async () => {
  const browser = await chromium.launch({headless: true});
  const page = await browser.newPage();
  await page.goto(pathToFileURL(path.join(__dirname, 'manual.html')).href);
  await page.evaluate(() => document.fonts.ready);
  const overflow = await page.locator('.page').evaluateAll(pages => pages.map((p,i) => ({page:i+1, overflow:p.scrollHeight>p.clientHeight})));
  if (overflow.some(p => p.overflow)) throw new Error(JSON.stringify(overflow));
  await page.pdf({path:path.resolve(__dirname, '../../output/pdf/epsx-user-guide-th.pdf'),printBackground:true,preferCSSPageSize:true});
  await browser.close();
})();
