import { Injectable } from "@nestjs/common";
import { FinancialConfig } from "../configs/financial.config";

interface CacheEntry<T> {
  data: T;
  timestamp: number;
}

@Injectable()
export class CacheService {
  private cache = new Map<string, CacheEntry<any>>();

  get<T>(key: string): T | null {
    const entry = this.cache.get(key);
    if (!entry) return null;

    if (Date.now() - entry.timestamp < FinancialConfig.cache.expiry) {
      return entry.data;
    }

    this.cache.delete(key);
    return null;
  }

  set<T>(key: string, data: T): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
    });
  }

  clear(): void {
    this.cache.clear();
  }
}
