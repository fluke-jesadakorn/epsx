import { chromium } from 'playwright';
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
console.log('Loading /...');
await page.goto('http://localhost:3000/', { waitUntil: 'networkidle', timeout: 25000 });
console.log('  loaded');
await page.waitForTimeout(1200);
const data = await page.evaluate(() => ({
  bodyLen: (document.body?.innerText || '').length,
  bodyFirst: (document.body?.innerText || '').slice(0, 300),
  h1: document.querySelectorAll('h1').length,
  title: document.title,
}));
console.log(JSON.stringify(data, null, 2));

console.log('Loading /dashboard...');
await page.goto('http://localhost:3000/dashboard', { waitUntil: 'networkidle', timeout: 25000 });
console.log('  loaded');
await page.waitForTimeout(1200);
const data2 = await page.evaluate(() => ({
  bodyLen: (document.body?.innerText || '').length,
  bodyFirst: (document.body?.innerText || '').slice(0, 300),
  h1: document.querySelectorAll('h1').length,
  title: document.title,
}));
console.log(JSON.stringify(data2, null, 2));

await browser.close();
