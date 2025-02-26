import { Controller, Get } from '@nestjs/common';
import { ApiOperation, ApiResponse, ApiTags } from '@nestjs/swagger';
import { HealthCheckResponse } from './types/market.types';
import { MarketService } from './market.service';

@ApiTags('Market')
@Controller('market')
export class MarketController {
  constructor(private readonly marketService: MarketService) {}

  @Get('health')
  @ApiOperation({
    summary: 'Check market service health',
    description: 'Returns the health status of the market service'
  })
  @ApiResponse({
    status: 200,
    description: 'Service is healthy',
    type: HealthCheckResponse,
    schema: {
      example: {
        status: 'ok',
        service: 'market'
      }
    }
  })
  @ApiResponse({
    status: 503,
    description: 'Service is unavailable'
  })
  async getHealth(): Promise<HealthCheckResponse> {
    return {
      status: 'ok',
      service: 'market',
    };
  }
}
