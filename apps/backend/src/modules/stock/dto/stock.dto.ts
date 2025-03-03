import { ApiProperty } from '@nestjs/swagger';
import { PaginationParams } from '@epsx/shared/types';

export class StockPaginationParamsDto implements PaginationParams {
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
}

export class StockResponseDto {
  @ApiProperty({
    description: 'Unique identifier of the stock',
    example: '507f1f77bcf86cd799439011'
  })
  _id!: string;

  @ApiProperty({
    description: 'Stock symbol',
    example: 'AAPL'
  })
  symbol!: string;

  @ApiProperty({
    description: 'Company name',
    example: 'Apple Inc.'
  })
  company_name!: string;

  @ApiProperty({
    description: 'Market cap in billions',
    example: 2850.12
  })
  market_cap?: number;

  @ApiProperty({
    description: 'Market sector',
    example: 'Technology'
  })
  sector?: string;

  @ApiProperty({
    description: 'Average trading volume',
    example: 80000000
  })
  volume?: number;

  @ApiProperty({
    description: 'Exchange reference',
    example: '507f1f77bcf86cd799439011'
  })
  exchange!: string;

  @ApiProperty({
    description: 'Creation timestamp',
    example: '2024-02-26T08:30:00.000Z'
  })
  createdAt?: Date;

  @ApiProperty({
    description: 'Last update timestamp',
    example: '2024-02-26T08:30:00.000Z'
  })
  updatedAt?: Date;
}

export class PaginatedStockResponse {
  @ApiProperty({
    description: 'List of stocks',
    type: [StockResponseDto]
  })
  data!: StockResponseDto[];

  @ApiProperty({
    description: 'Total number of records',
    example: 100
  })
  total!: number;

  @ApiProperty({
    description: 'Current page number',
    example: 1
  })
  page!: number;

  @ApiProperty({
    description: 'Items per page',
    example: 20
  })
  limit!: number;

  @ApiProperty({
    description: 'Total number of pages',
    example: 5
  })
  totalPages!: number;
}

export class ScrapeByMarketCapDto {
  @ApiProperty({
    description: 'Minimum market cap in billions',
    example: 1,
    required: false
  })
  minMarketCap?: number;

  @ApiProperty({
    description: 'Maximum market cap in billions',
    example: 100,
    required: false
  })
  maxMarketCap?: number;
}

export class ScrapingStatusDto {
  @ApiProperty({
    description: 'Status of the scraping operation',
    example: 'completed'
  })
  status!: string;

  @ApiProperty({
    description: 'Number of records processed',
    example: 500
  })
  processed!: number;

  @ApiProperty({
    description: 'Number of records failed',
    example: 0
  })
  failed!: number;

  @ApiProperty({
    description: 'Total number of records',
    example: 500
  })
  total!: number;

  @ApiProperty({
    description: 'Error message if any',
    example: 'Failed to fetch data from API',
    required: false
  })
  error?: string;
}
