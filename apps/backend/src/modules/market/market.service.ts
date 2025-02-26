import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { HttpService } from '@nestjs/axios';

@Injectable()
export class MarketService {
  constructor(
    private readonly configService: ConfigService,
    private readonly httpService: HttpService,
  ) {}

  async getHealth() {
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
    };
  }

  // Add more market-related service methods here
}
