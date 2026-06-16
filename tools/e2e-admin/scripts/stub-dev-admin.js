// stub-dev-admin.js — tiny local HTTP server that mimics the dev admin BFF
// for harness smoke testing. It returns 404 + a minimal HTML body for any
// request, which lets the e2e-admin diff infra run end-to-end when the real
// K8s dev admin pod is down (ImagePullBackOff, etc.).
//
// Usage:
//   node scripts/stub-dev-admin.js [--port 3001] [--status 404]
//
// Then point the dev capture at it:
//   EPSX_DEV_BASE=http://localhost:3001 bash tools/e2e-admin/capture-dev-admin.sh
//
// The HTML body is a fixed placeholder so the harness can record
// "dev BFF is up but the route is empty" — this is the expected state
// for the wave 24 admin T1' initial pass.

const http = require("http");

const args = (() => {
  const a = {};
  for (let i = 2; i < process.argv.length; i += 2) {
    a[process.argv[i].replace(/^--/, "").replace(/-/g, "_")] = process.argv[i + 1];
  }
  return a;
})();

const PORT = parseInt(args.port || "3001", 10);
const STATUS = parseInt(args.status || "404", 10);

const HTML_404 = `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>404 — admin dev shell empty</title>
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <style>
    body { font-family: -apple-system, system-ui, sans-serif; max-width: 720px; margin: 80px auto; padding: 0 24px; color: #1a1a1a; }
    h1 { font-size: 28px; margin-bottom: 8px; }
    p { color: #555; line-height: 1.5; }
    code { background: #f3f3f3; padding: 2px 6px; border-radius: 4px; }
  </style>
</head>
<body>
  <h1>404 — admin dev shell empty</h1>
  <p>This is the wave-24 T1' stub for <code>tools/e2e-admin</code>.</p>
  <p>The Dioxus admin frontend is not yet wired up. Real routes will be
     added by T2 / T3 / T4 wave-24 tracks.</p>
  <p>Stub server: <code>scripts/stub-dev-admin.js</code></p>
</body>
</html>
`;

const server = http.createServer((req, res) => {
  console.log(`[stub] ${req.method} ${req.url} -> ${STATUS}`);
  res.writeHead(STATUS, { "content-type": "text/html; charset=utf-8" });
  res.end(HTML_404);
});

server.listen(PORT, () => {
  console.log(`stub-dev-admin listening on http://localhost:${PORT} (all routes -> ${STATUS})`);
});
