import { Controller, Get, Query, UseGuards, Req } from '@nestjs/common';
import { ApiOperation, ApiQuery, ApiResponse, ApiTags } from '@nestjs/swagger';
import { HealthCheckResponse } from './types/market.types';
import { MarketService } from './market.service';
import { EpsGrowth, Paginate } from '@epsx/shared';
import { RolesGuard, Roles, UserRole } from '../../shared/guards/role.guard';
import type { Request } from 'express';

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

  @Get('financials/eps-growth')
  @UseGuards(RolesGuard)
  @Roles(UserRole.PUBLIC)
  @ApiOperation({
    summary: 'Get EPS growth data with pagination',
    description: 'Returns paginated list of companies EPS growth data sorted by EPS growth percentage. Access levels: Public (rank >21), Basic (rank >11), Premium (all ranks)'
  })
  @ApiQuery({
    name: 'limit',
    required: false,
    type: Number,
    description: 'Number of records to return',
    example: 10
  })
  @ApiQuery({
    name: 'skip',
    required: false,
    type: Number,
    description: 'Number of records to skip',
    example: 0
  })
  @ApiResponse({
    status: 200,
    description: 'Successfully retrieved EPS growth data',
    schema: {
      type: 'object',
      properties: {
        data: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              symbol: { type: 'string', example: 'AAPL' },
              company_name: { type: 'string', example: 'Apple Inc.' },
              market_code: { type: 'string', example: 'SET' },
              eps_diluted: { type: 'number', example: 6.42 },
              previous_eps_diluted: { type: 'number', example: 5.55 },
              eps_growth: { type: 'number', example: 15.7 },
              report_date: { type: 'string', format: 'date-time', example: '2024-02-26T08:30:00.000Z' },
              year: { type: 'number', example: 2024 },
              quarter: { type: 'number', example: 4 }
            }
          }
        },
        total: { type: 'number', example: 100, description: 'Total number of records' },
        page: { type: 'number', example: 1, description: 'Current page number' },
        limit: { type: 'number', example: 10, description: 'Number of records per page' }
      }
    }
  })
  async getEpsGrowth(
    @Query('limit') limit: number = 10,
    @Query('skip') skip: number = 0,
    @Req() request: Request & { userRole?: UserRole }
  ): Promise<Paginate<EpsGrowth>> {
    return this.marketService.getEpsGrowth(skip, limit, request.userRole || UserRole.PUBLIC);
  }
}
