import { spawn } from "bun";

const ANVIL_PORT = 8546;
const PROXY_PORT = 8545;

console.log(`\n🔌 Starting Anvil Proxy...`);
console.log(`   Proxy: http://0.0.0.0:${PROXY_PORT} (accessible via Tailscale)`);
console.log(`   Anvil: http://127.0.0.1:${ANVIL_PORT} (background)`);

// Start Anvil on background port
const anvil = spawn(["./apps/contracts/start-anvil-local.sh"], {
    env: { ...process.env, ANVIL_PORT: ANVIL_PORT.toString() },
    stdout: "inherit",
    stderr: "inherit",
});

// Handle cleanup
process.on("SIGINT", () => {
    anvil.kill();
    process.exit();
});

// Simple Transaction Viewer HTML
const getHtml = (txHash: string) => `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Transaction Details | Local Anvil</title>
    <style>
        :root { --bg: #0f111a; --card: #1e212b; --text: #e2e8f0; --accent: #3b82f6; }
        body { background: var(--bg); color: var(--text); font-family: system-ui, -apple-system, sans-serif; padding: 2rem; max-width: 800px; margin: 0 auto; line-height: 1.5; }
        .container { background: var(--card); border-radius: 12px; padding: 2rem; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1); }
        h1 { margin-top: 0; font-size: 1.5rem; border-bottom: 1px solid #334155; padding-bottom: 1rem; }
        .row { display: flex; padding: 0.75rem 0; border-bottom: 1px solid #334155; }
        .row:last-child { border-bottom: none; }
        .label { width: 150px; color: #94a3b8; font-weight: 500; }
        .value { flex: 1; font-family: monospace; word-break: break-all; }
        .tag { padding: 2px 8px; border-radius: 4px; font-size: 0.8rem; font-weight: bold; }
        .success { background: rgba(34, 197, 94, 0.2); color: #4ade80; }
        .failed { background: rgba(239, 68, 68, 0.2); color: #f87171; }
        .loading { text-align: center; padding: 2rem; color: #94a3b8; }
        a { color: var(--accent); text-decoration: none; }
        a:hover { text-decoration: underline; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Transaction Details</h1>
        <div id="content" class="loading">Loading transaction...</div>
        <div style="margin-top: 1rem; font-size: 0.8rem; color: #64748b;">
            Hash: ${txHash}
        </div>
    </div>
    <script>
        const txHash = "${txHash}";
        
        async function fetchRpc(method, params) {
            const res = await fetch("/", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ jsonrpc: "2.0", method, params, id: 1 })
            });
            const data = await res.json();
            return data.result;
        }

        async function load() {
            try {
                // If the hash is invalid/short, this might return null or error
                const tx = await fetchRpc("eth_getTransactionByHash", [txHash]);
                if (!tx) {
                    document.getElementById("content").innerHTML = \`
                        <div class="row"><span class="value" style="color: #f87171">Transaction not found</span></div>
                        <p style="text-align: center; color: #64748b; margin-top: 1rem;">
                            Ensure the transaction hash is correct and exists on this local Anvil chain (Chain ID 31337).
                        </p>\`;
                    return;
                }
                
                const receipt = await fetchRpc("eth_getTransactionReceipt", [txHash]);
                const status = receipt ? (receipt.status === "0x1" ? "Success" : "Failed") : "Pending";
                const statusClass = receipt ? (receipt.status === "0x1" ? "success" : "failed") : "";

                const rows = [
                    { label: "Hash", value: tx.hash },
                    { label: "Status", value: \`<span class="tag \${statusClass}">\${status}</span>\` },
                    { label: "Block", value: parseInt(tx.blockNumber, 16) || "Pending" },
                    { label: "From", value: tx.from },
                    { label: "To", value: tx.to || "Contract Creation" },
                    { label: "Value", value: (parseInt(tx.value, 16) / 1e18) + " ETH" },
                    { label: "Gas Used", value: receipt ? parseInt(receipt.gasUsed, 16) : "Pending" },
                    { label: "Gas Price", value: (parseInt(tx.gasPrice, 16) / 1e9) + " Gwei" },
                    { label: "Nonce", value: parseInt(tx.nonce, 16) }
                ];

                document.getElementById("content").innerHTML = rows.map(r => 
                    \`<div class="row"><div class="label">\${r.label}</div><div class="value">\${r.value}</div></div>\`
                ).join("");
            } catch (e) {
                document.getElementById("content").innerText = "Error loading transaction: " + e.message;
            }
        }
        load();
    </script>
</body>
</html>
`;

// Start Proxy Server
Bun.serve({
    hostname: "0.0.0.0",
    port: PROXY_PORT,
    async fetch(req) {
        const url = new URL(req.url);

        // Serve Transaction Viewer HTML for /tx/[hash]
        // MATCHES ANY /tx/* PATH to prevent 404s on user typos
        if (req.method === "GET" && /^\/tx\/.+/.test(url.pathname)) {
            // Extract hash or whatever follows /tx/
            const txHash = url.pathname.replace(/^\/tx\//, "").replace(/\/$/, "");
            return new Response(getHtml(txHash), {
                headers: { "Content-Type": "text/html" }
            });
        }

        // Forward everything else to Anvil RPC
        try {
            const anvilUrl = `http://127.0.0.1:${ANVIL_PORT}${url.pathname}${url.search}`;

            const init: RequestInit = {
                method: req.method,
                headers: req.headers,
                body: req.body,
            };
            // @ts-ignore
            if (req.body) init.duplex = "half";

            const response = await fetch(anvilUrl, init);
            return response;
        } catch (e) {
            return new Response("Anvil proxy error: Is Anvil running?", { status: 502 });
        }
    },
});
