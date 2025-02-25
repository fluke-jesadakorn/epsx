import { Injectable, Logger } from "@nestjs/common";
import axios, { AxiosInstance } from "axios";

interface FetchConfig {
  maxRetries: number;
  retryDelay: number;
  timeout: number;
  headers?: Record<string, string>;
}

@Injectable()
export class HttpService {
  private readonly logger = new Logger(HttpService.name);
  private readonly axiosInstance: AxiosInstance;
  private readonly defaultConfig: FetchConfig;

  constructor() {
    this.defaultConfig = {
      maxRetries: 3,
      retryDelay: 1000,
      timeout: 30000,
      headers: {
        "Content-Type": "application/json",
      },
    };

    this.axiosInstance = axios.create({
      timeout: this.defaultConfig.timeout,
      headers: this.defaultConfig.headers,
    });

    // Add request interceptor for logging
    this.axiosInstance.interceptors.request.use(
      (config) => {
        this.logger.debug(`Making request to ${config.url}`);
        return config;
      },
      (error) => {
        this.logger.error("Request error:", error);
        return Promise.reject(error);
      }
    );

    // Add response interceptor for logging
    this.axiosInstance.interceptors.response.use(
      (response) => {
        this.logger.debug(`Received response from ${response.config.url}`);
        return response;
      },
      (error) => {
        this.logger.error("Response error:", error);
        return Promise.reject(error);
      }
    );
  }

  private async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private async fetch<T>(
    url: string,
    config: Partial<FetchConfig> = {},
    retryCount = 0
  ): Promise<T | null> {
    const finalConfig = { ...this.defaultConfig, ...config };

    try {
      this.logger.debug(`Fetching data from: ${url}`);
      const response = await this.axiosInstance.get(url, {
        timeout: finalConfig.timeout,
        headers: finalConfig.headers,
      });

      if (response.status === 429 && retryCount < finalConfig.maxRetries) {
        const dynamicDelay = finalConfig.retryDelay * Math.pow(2, retryCount);
        this.logger.warn(`Rate limited, retrying in ${dynamicDelay}ms...`);
        await this.sleep(dynamicDelay);
        return this.fetch(url, config, retryCount + 1);
      }

      if (response.status < 200 || response.status >= 300) {
        this.logger.warn(`Failed to fetch data: ${response.statusText}`);
        return null;
      }

      return response.data;
    } catch (error) {
      const isAxiosError = axios.isAxiosError(error);
      const errorMessage = isAxiosError ? error.message : String(error);
      this.logger.error(`Error fetching data: ${errorMessage}`);

      if (retryCount < finalConfig.maxRetries) {
        const dynamicDelay = finalConfig.retryDelay * Math.pow(2, retryCount);
        this.logger.warn(
          `Retrying in ${dynamicDelay}ms (attempt ${retryCount + 1})...`
        );
        await this.sleep(dynamicDelay);
        return this.fetch(url, config, retryCount + 1);
      }

      return null;
    }
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
