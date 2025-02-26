import { Controller, Get } from '@nestjs/common';
import { ApiTags } from '@nestjs/swagger';
import { MarketService } from './market.service';

@ApiTags('Market')
@Controller('market')
export class MarketController {
  constructor(private readonly marketService: MarketService) {}

  @Get('health')
  async getHealth() {
    return {
      status: 'ok',
      service: 'market',
    };
  }
}
