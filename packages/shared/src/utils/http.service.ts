import { Injectable, Logger } from '@nestjs/common';
import axios, { AxiosInstance, AxiosError } from 'axios';

export class HttpServiceError extends Error {
  status: number;
  cause?: Error;

  constructor(message: string, status: number = 500, cause?: Error) {
    super(message);
    this.name = 'HttpServiceError';
    this.status = status;
    this.cause = cause;
  }
}

export interface FetchConfig {
  maxRetries: number;
  retryDelay: number;
  timeout: number;
  headers?: Record<string, string>;
}

@Injectable()
export class BaseHttpService {
  protected readonly logger = new Logger(this.constructor.name);
  protected readonly axiosInstance: AxiosInstance;
  protected readonly defaultConfig: FetchConfig;

  constructor() {
    this.defaultConfig = {
      maxRetries: 3,
      retryDelay: 1000,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
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
        this.logger.error('Request error:', error);
        return Promise.reject(error);
      },
    );

    // Add response interceptor for logging
    this.axiosInstance.interceptors.response.use(
      (response) => {
        this.logger.debug(`Received response from ${response.config.url}`);
        return response;
      },
      (error) => {
        this.logger.error('Response error:', error);
        return Promise.reject(error);
      },
    );
  }

  protected async sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Makes an HTTP request with built-in retry logic and exponential backoff
   */
  protected async fetch<T>(
    url: string,
    config: Partial<FetchConfig> = {},
    retryCount = 0,
  ): Promise<T | null> {
    const finalConfig = { ...this.defaultConfig, ...config };

    try {
      this.logger.debug(`Fetching data from: ${url}`);
      const response = await this.axiosInstance.get(url, {
        timeout: finalConfig.timeout,
        headers: finalConfig.headers,
      });

      // Handle rate limiting
      if (response.status === 429 && retryCount < finalConfig.maxRetries) {
        const dynamicDelay = finalConfig.retryDelay * Math.pow(2, retryCount);
        this.logger.warn(`Rate limited, retrying in ${dynamicDelay}ms...`);
        await this.sleep(dynamicDelay);
        return this.fetch(url, config, retryCount + 1);
      }

      if (response.status < 200 || response.status >= 300) {
        throw new HttpServiceError(
          `HTTP ${response.status}: ${response.statusText}`,
          response.status
        );
      }

      return response.data;
    } catch (error) {
      const isAxiosError = axios.isAxiosError(error);
      const errorMessage = isAxiosError ? error.message : String(error);
      this.logger.error(`Error fetching data: ${errorMessage}`);

      if (retryCount < finalConfig.maxRetries) {
        const dynamicDelay = finalConfig.retryDelay * Math.pow(2, retryCount);
        this.logger.warn(`Retrying in ${dynamicDelay}ms (attempt ${retryCount + 1})...`);
        await this.sleep(dynamicDelay);
        return this.fetch(url, config, retryCount + 1);
      }

      // Create a detailed error object
      throw new HttpServiceError(
        errorMessage,
        isAxiosError && error.response ? error.response.status : 500,
        error as Error
      );
    }
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
   */
}
