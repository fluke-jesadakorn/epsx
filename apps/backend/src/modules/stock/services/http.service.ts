import { Injectable, Logger } from '@nestjs/common';
import axios, { AxiosInstance } from 'axios';
import { 
  IHttpService, 
  IHttpServiceResponse, 
  IStockScreenerResponse
} from '../types';

@Injectable()
export class HttpService implements IHttpService {
  private readonly logger = new Logger(HttpService.name);
  private readonly client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: process.env.STOCK_API_URL,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      }
    });

    // Add request interceptor for logging
    this.client.interceptors.request.use(
      (config) => {
        this.logger.debug(`Making request to ${config.url}`);
        return config;
      },
      (error) => {
        this.logger.error('Request error:', error);
        return Promise.reject(error);
      }
    );

    // Add response interceptor for logging
    this.client.interceptors.response.use(
      (response) => {
        this.logger.debug(`Received response from ${response.config.url}`);
        return response;
      },
      (error) => {
        this.logger.error('Response error:', error);
        return Promise.reject(error);
      }
    );
  }

  async fetchStockScreener(marketCode: string): Promise<IHttpServiceResponse<IStockScreenerResponse>> {
    try {
      let endpoint: string;
      let params: Record<string, string> = {};

      // Handle different market types
      switch (marketCode) {
        case 'NASDAQ':
          endpoint = '/screener/s';
          params = {
            m: 'marketCap',
            s: 'desc',
            c: 's,n',
            f: 'exchange-is-NASDAQ'
          };
          break;
        case 'NYSE':
          endpoint = '/screener/a';
          params = {
            m: 'marketCap',
            s: 'desc',
            c: 's,n',
            f: 'exchangeCode-is-NYSE'
          };
          break;
        case 'UKR':
          endpoint = '/screener/u';
          params = {
            m: 'marketCap',
            s: 'desc',
            c: 's,n',
            f: 'exchangeCode-is-UKR'
          };
          break;
        default:
          this.logger.warn(`Unsupported market code: ${marketCode}`);
          throw new Error(`Market ${marketCode} is not supported`);
      }

      const response = await this.client.get(endpoint, { params });
      return {
        data: response.data,
        status: response.status,
        statusText: response.statusText,
        headers: response.headers as Record<string, string>
      };
    } catch (error) {
      this.logger.error(`Failed to fetch stock screener data for market ${marketCode}:`, error);
      throw error;
    }
  }
}
