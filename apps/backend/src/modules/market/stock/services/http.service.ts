import { Injectable } from "@nestjs/common";
import { BaseHttpService } from "@epsx/nest-core";

@Injectable()
export class HttpService extends BaseHttpService {
  constructor() {
    super(HttpService.name);
  }

  /**
   * Makes a request to the Stock Analysis Screener API
   * Uses different endpoints based on market type:
   * - NASDAQ uses 's' endpoint with 'exchange' filter
   * - Other markets use 'a' endpoint with 'exchangeCode' filter
   *
   * Future improvements:
   * - Add market type validation
   * - Support custom filter parameters
   * - Cache market-specific endpoint configurations
   * - Add request rate limiting per market
   */
  async fetchStockScreener<T>(market: string): Promise<T | null> {
    const endpoint = market === "NASDAQ" ? "s" : "a";
    const filterParam = market === "NASDAQ" ? "exchange" : "exchangeCode";
    const url = `https://api.stockanalysis.com/api/screener/${endpoint}/f?m=marketCap&s=desc&c=s,n&f=${filterParam}-is-${market}`;

    // Add X-Source header for specific stock analysis requests
    return this.fetch<T>(url, {
      headers: {
        ...this.defaultConfig.headers,
        "X-Source": "API-Client",
      },
    });
  }

  /**
   * Future Features To Implement:
   * 1. Add request queueing system for better rate limiting
   * 2. Implement circuit breaker pattern for failing endpoints
   * 3. Add request caching layer
   * 4. Support different retry strategies per endpoint
   * 5. Add metrics collection for request performance
   * 6. Implement request priority system
   * 7. Add support for different authentication methods
   * 8. Implement request batching for similar endpoints
   *
   * Planned API Methods:
   * - Real-time price updates API
   * - Historical data API
   * - Market indicators API
   * - Company fundamentals API
   */
}
