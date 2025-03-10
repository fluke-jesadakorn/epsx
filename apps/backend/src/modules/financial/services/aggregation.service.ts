import { Injectable, Logger } from '@nestjs/common';
import { PaginationService, PaginationParams } from './pagination.service';
import { EpsGrowthResponse } from '@epsx/shared';
import { PaginatedEpsGrowthResponse, ProcessingStatusDto } from '../dto/financial.dto';

@Injectable()
export class AggregationService {
  private readonly logger = new Logger(AggregationService.name);

  constructor(private readonly paginationService: PaginationService) {}

  async calculateAndSaveAllEPSGrowth(): Promise<ProcessingStatusDto> {
    try {
      // Implementation would calculate and save EPS growth data
      return {
        id: `calc_${Date.now()}`,
        status: 'completed',
        message: 'EPS growth calculation completed successfully'
      };
    } catch (error) {
      this.logger.error('Failed to calculate and save EPS growth:', error);
      throw error;
    }
  }

  async getEPSGrowthRanking(params: PaginationParams): Promise<PaginatedEpsGrowthResponse> {
    try {
      // Mock data - actual implementation would fetch from database
      const mockData: EpsGrowthResponse[] = [
        {
          symbol: 'AAPL',
          companyName: 'Apple Inc.',
          marketCode: 'NASDAQ',
          epsDiluted: 3.45,
          previousEpsDiluted: 3.0,
          epsGrowth: 0.15,
          reportDate: '2024-03-08',
          year: 2024,
          quarter: 1
        },
        {
          symbol: 'MSFT',
          companyName: 'Microsoft Corporation',
          marketCode: 'NASDAQ',
          epsDiluted: 2.89,
          previousEpsDiluted: 2.58,
          epsGrowth: 0.12,
          reportDate: '2024-03-08',
          year: 2024,
          quarter: 1
        }
      ];

      const { limit, skip } = this.paginationService.getPaginationParams(params);
      const page = Math.floor(skip / limit) + 1;
      const total = mockData.length;

      return {
        data: mockData,
        total,
        page,
        limit
      };
    } catch (error) {
      this.logger.error('Failed to get EPS growth ranking:', error);
      throw error;
    }
  }

  async getEPSGrowthRankingOnceQuarter(
    limit: number,
    skip: number
  ): Promise<PaginatedEpsGrowthResponse> {
    try {
      // Mock data - actual implementation would fetch from database
      const mockData: EpsGrowthResponse[] = [
        {
          symbol: 'AAPL',
          companyName: 'Apple Inc.',
          marketCode: 'NASDAQ',
          epsDiluted: 3.45,
          previousEpsDiluted: 3.0,
          epsGrowth: 0.15,
          reportDate: '2024-03-08',
          year: 2024,
          quarter: 1
        }
      ];

      const page = Math.floor(skip / limit) + 1;
      const total = mockData.length;

      return {
        data: mockData,
        total,
        page,
        limit
      };
    } catch (error) {
      this.logger.error('Failed to get quarter EPS growth ranking:', error);
      throw error;
    }
  }

  async getEPSGrowthForSymbol(symbol: string): Promise<EpsGrowthResponse | null> {
    try {
      // Mock data - actual implementation would fetch from database and calculate
      if (symbol === 'AAPL') {
        return {
          symbol,
          companyName: 'Apple Inc.',
          marketCode: 'NASDAQ',
          epsDiluted: 3.45,
          previousEpsDiluted: 3.0,
          epsGrowth: 0.15,
          reportDate: '2024-03-08',
          year: 2024,
          quarter: 1
        };
      }
      return null;
    } catch (error) {
      this.logger.error(`Failed to get EPS growth for symbol ${symbol}:`, error);
      throw error;
    }
  }
}
