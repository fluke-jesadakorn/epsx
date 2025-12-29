import { sleep } from "bun";

const API_URL = "http://localhost:8080/api/v1/developer-portal/stats";
// Using an endpoint that hits the middleware stack

const TOTAL_REQUESTS = 70;
const CONCURRENCY = 1;

async function makeRequest(id: number, apiKey?: string, wallet?: string) {
    const headers: Record<string, string> = {};
    if (wallet) {
        headers["X-Wallet-Address"] = wallet;
    }
    if (apiKey) {
        headers["x-api-key"] = apiKey;
    }

    const start = performance.now();
    try {
        const res = await fetch(API_URL, { headers });
        const end = performance.now();
        const duration = (end - start).toFixed(2);

        // Get rate limit headers
        const limit = res.headers.get("X-RateLimit-Limit") || res.headers.get("X-RateLimit-Limit-Global") || "N/A";
        const remaining = res.headers.get("X-RateLimit-Remaining") || res.headers.get("X-RateLimit-Remaining-Global") || "N/A";

        const statusColor = res.status === 429 ? "\x1b[31m" : res.status >= 200 && res.status < 300 ? "\x1b[32m" : "\x1b[33m";
        const resetColor = "\x1b[0m";

        console.log(
            `${statusColor}[Req ${id}] Status: ${res.status} | Time: ${duration}ms | Limit: ${limit} | Remaining: ${remaining}${resetColor}`
        );

        return res.status;
    } catch (err) {
        console.log(`[Req ${id}] Error: ${err.message}`);
        return 0;
    }
}

async function runTest(name: string, apiKey?: string, wallet?: string, requests = TOTAL_REQUESTS) {
    console.log(`\n\n--- Starting Test: ${name} ---`);
    if (wallet) console.log(`Wallet: ${wallet}`);
    if (apiKey) console.log(`API Key: ${apiKey.substring(0, 8)}...`);

    let completed = 0;
    let rateLimited = 0;
    let success = 0;

    // Run in batches
    for (let i = 0; i < requests; i += CONCURRENCY) {
        const batch = [];
        for (let j = 0; j < CONCURRENCY && i + j < requests; j++) {
            batch.push(makeRequest(i + j + 1, apiKey, wallet));
        }

        const results = await Promise.all(batch);
        results.forEach(status => {
            if (status === 429) rateLimited++;
            if (status >= 200 && status < 300) success++;
        });

        // Small delay
        await sleep(100);
    }

    console.log(`\nTest Complete: ${name}`);
    console.log(`Total: ${requests}`);
    console.log(`Success (2xx): ${success}`);
    console.log(`Rate Limited (429): ${rateLimited}`);
}

async function main() {
    const apiKey = process.env.TEST_API_KEY;
    const wallet = process.env.TEST_WALLET || "0x71bE63f3384f5fb98995898A86B02Fb2426c5788"; // Default test wallet

    console.log("🚀 Starting Usage & Rate Limit Verification Script");

    if (!apiKey) {
        console.warn("\x1b[33mWARNING: No TEST_API_KEY environment variable provided.\x1b[0m");
        console.warn("Usage counting will NOT work without a valid API Key ID (UUID).");
        console.warn("Example: TEST_API_KEY=your-uuid-here bun run scripts/test-rate-limit.ts");
    }

    // Test with API Key if available
    await runTest("API Key Usage Check", apiKey, wallet);
}

main();
