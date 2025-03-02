import { Injectable, Logger } from '@nestjs/common';
import axios, { AxiosInstance } from 'axios';
import { IHttpService, IHttpServiceResponse, IStockScreenerData } from '../types';

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

  async fetchStockScreener(marketCode: string): Promise<IHttpServiceResponse<IStockScreenerData>> {
    try {
      const response = await this.client.get(`/screener/${marketCode}`);
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
