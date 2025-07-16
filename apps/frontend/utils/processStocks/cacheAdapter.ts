import type { FinancialsWithCurrentPrice } from './getPriceAndEps';

// ---------- 1. In-memory driver (today) ----------
const memoryStore = new Map<string, FinancialsWithCurrentPrice>();
const memoryDriver = {
  async set(key: string, val: FinancialsWithCurrentPrice) { memoryStore.set(key, val); },
  async get(key: string) { return memoryStore.get(key) ?? null; },
  async clear() { memoryStore.clear(); },
  async entries() { return Array.from(memoryStore.entries()); },
};

// ---------- 2. Redis driver (future) ----------
// import Redis from 'ioredis';
// const redis = new Redis(process.env.REDIS_URL);
// const redisDriver = {
//   async set(key: string, val: FinancialsWithCurrentPrice) {
//     await redis.set(key, JSON.stringify(val), 'EX', 86400);
//   },
//   async get(key: string) {
//     const raw = await redis.get(key);
//     return raw ? JSON.parse(raw) : null;
//   },
//   async clear() { await redis.flushdb(); },
//   async entries() { /* scan or use a dedicated hash */ return []; },
// };

// ---------- 3. Switch here ----------
const driver = memoryDriver;   // <-- change to redisDriver when ready
// ------------------------------------

export const cache = {
  async saveTop50(data: Record<string, FinancialsWithCurrentPrice>) {
    await driver.clear();
    await Promise.all(Object.entries(data).map(([k, v]) => driver.set(k, v)));
  },
  async getTop50(): Promise<Record<string, FinancialsWithCurrentPrice>> {
    const pairs = await driver.entries();
    return Object.fromEntries(pairs);
  },
};
