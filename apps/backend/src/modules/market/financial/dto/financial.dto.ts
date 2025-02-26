import { ApiProperty } from '@nestjs/swagger';
import { EpsGrowthData } from '../services/aggregation.service';
import { PaginatedResponse, PaginationMetadata } from '../types/financial.types';

export class ProcessingStatusDto {
  @ApiProperty({
    description: 'Status of the EPS growth processing',
    example: 'completed'
  })
  status!: string;

  @ApiProperty({
    description: 'Processing progress percentage',
    example: 100
  })
  progress?: number;

  @ApiProperty({
    description: 'Error message if processing failed',
    example: 'Failed to fetch data from API'
  })
  error?: string;
}

export class GetEPSGrowthRankingDto {
  @ApiProperty({
    description: 'Number of items per page',
    example: 20,
    required: false
  })
  limit?: number;

  @ApiProperty({
    description: 'Number of items to skip',
    example: 0,
    required: false
  })
  skip?: number;

  @ApiProperty({
    description: 'Market code to filter results',
    example: 'SET',
    required: false
  })
  market_code?: string;

  @ApiProperty({
    description: 'Field to sort by',
    example: 'eps_growth',
    required: false
  })
  sortBy?: string;

  @ApiProperty({
    description: 'Sort order',
    enum: ['asc', 'desc'],
    example: 'desc',
    required: false
  })
  sortOrder?: 'asc' | 'desc';
}

export class EpsGrowthResponseDto implements EpsGrowthData {
  @ApiProperty({
    description: 'Stock symbol',
    example: 'AAPL'
  })
  symbol!: string;

  @ApiProperty({
    description: 'Market code',
    example: 'SET'
  })
  market_code!: string;

  @ApiProperty({
    description: 'Company name',
    example: 'Apple Inc.'
  })
  company_name!: string;

  @ApiProperty({
    description: 'Diluted EPS',
    example: 6.42
  })
  eps_diluted!: number;

  @ApiProperty({
    description: 'Previous diluted EPS',
    example: 5.55
  })
  previous_eps_diluted!: number;

  @ApiProperty({
    description: 'EPS growth percentage',
    example: 15.7
  })
  eps_growth!: number;

  @ApiProperty({
    description: 'Report date',
    example: '2024-02-26T08:30:00.000Z'
  })
  report_date!: string;

  @ApiProperty({
    description: 'Fiscal year',
    example: 2024
  })
  year!: number;

  @ApiProperty({
    description: 'Fiscal quarter',
    example: 4
  })
  quarter!: number;
}

export class PaginatedEpsGrowthResponse implements PaginatedResponse<EpsGrowthData> {
  @ApiProperty({
    description: 'List of EPS growth data',
    type: [EpsGrowthResponseDto]
  })
  data!: EpsGrowthResponseDto[];

  @ApiProperty({
    description: 'Pagination metadata',
    type: 'object',
    properties: {
      total: {
        type: 'number',
        description: 'Total number of records',
        example: 100
      },
      limit: {
        type: 'number',
        description: 'Number of items per page',
        example: 20
      },
      skip: {
        type: 'number',
        description: 'Number of items skipped',
        example: 20
      },
      hasMore: {
        type: 'boolean',
        description: 'Whether there are more items available',
        example: true
      }
    }
  })
  metadata!: PaginationMetadata;
}
