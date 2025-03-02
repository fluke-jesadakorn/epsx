import { Injectable } from "@nestjs/common";
import { BaseHttpService } from "@epsx/nest-core";

@Injectable()
export class HttpService extends BaseHttpService {
  constructor() {
    super(HttpService.name);
  }

  async fetchStockAnalysis<T>(path: string): Promise<T | null> {
    const baseUrl = "https://stockanalysis.com";
    const url = `${baseUrl}${path}`;

    return this.fetch<T>(url, {
      headers: {
        ...this.defaultConfig.headers,
        "X-Source": "WebClient",
      },
    });
  }

  // TODO: Add support for:
  // - Request queueing for better rate limiting
  // - Circuit breaker pattern
  // - Request caching
  // - Metrics collection
  // - Request batching
}
