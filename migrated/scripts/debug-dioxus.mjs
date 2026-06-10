import { chromium } from 'playwright';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.goto('http://localhost:4000/', { waitUntil: 'networkidle', timeout: 20000 });
await page.waitForTimeout(1500);
const data = await page.evaluate(() => {
  return {
    title: document.title,
    bodyLen: (document.body?.innerText || '').length,
    bodyFirst: (document.body?.innerText || '').slice(0, 800),
    h1Count: document.querySelectorAll('h1').length,
    h1Text: Array.from(document.querySelectorAll('h1')).map(h => h.textContent?.slice(0, 60)),
  };
});
console.log(JSON.stringify(data, null, 2));
await browser.close();
