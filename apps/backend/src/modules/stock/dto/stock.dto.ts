import { ApiProperty } from '@nestjs/swagger';

export class StockPaginationParamsDto {
  @ApiProperty({
    description: 'Maximum number of stock records to return per page',
    type: 'number',
    minimum: 1,
    maximum: 100,
    default: 10,
    required: false,
    example: 20
  })
  limit?: number;

  @ApiProperty({
    description: 'Number of records to skip for pagination',
    type: 'number',
    minimum: 0,
    default: 0,
    required: false,
    example: 20
  })
  skip?: number;

  @ApiProperty({
    description: 'Field to sort results by',
    enum: ['marketCap', 'symbol', 'name', 'sector'],
    default: 'marketCap',
    required: false,
    example: 'marketCap'
  })
  sortBy?: string;

  @ApiProperty({
    description: 'Sort direction',
    enum: ['asc', 'desc'],
    default: 'desc',
    required: false,
    example: 'desc'
  })
  sortOrder?: 'asc' | 'desc';
}

export class StockResponseDto {
  @ApiProperty({
    description: 'Stock market symbol/ticker',
    example: 'AAPL',
    type: 'string',
    minLength: 1,
    maxLength: 5,
    pattern: '^[A-Z]{1,5}$'
  })
  symbol!: string;

  @ApiProperty({
    description: 'Company legal name',
    example: 'Apple Inc.',
    type: 'string',
    minLength: 1,
    maxLength: 100
  })
  name!: string;

  @ApiProperty({
    description: 'Stock exchange where the stock is listed',
    example: 'NASDAQ',
    type: 'string',
    minLength: 2,
    maxLength: 10
  })
  exchange!: string;

  @ApiProperty({
    description: 'Company market capitalization in USD',
    example: 2500000000000,
    type: 'number',
    minimum: 0,
    format: 'int64'
  })
  marketCap!: number;

  @ApiProperty({
    description: 'Business sector classification',
    example: 'Technology',
    type: 'string'
  })
  sector!: string;

  @ApiProperty({
    description: 'Specific industry within the sector',
    example: 'Consumer Electronics',
    type: 'string'
  })
  industry!: string;

  @ApiProperty({
    description: 'Company official website URL',
    example: 'https://www.apple.com',
    type: 'string',
    format: 'uri',
    pattern: '^https?://[\\w\\-]+(\\.[\\w\\-]+)+[/#?]?.*$'
  })
  website!: string;

  @ApiProperty({
    description: 'Brief company description and business overview',
    example: 'Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide.',
    type: 'string',
    maxLength: 2000
  })
  description!: string;

  @ApiProperty({
    description: 'Current Chief Executive Officer',
    example: 'Tim Cook',
    type: 'string'
  })
  ceo!: string;

  @ApiProperty({
    description: 'Total number of employees',
    example: 147000,
    type: 'number',
    minimum: 0,
    format: 'int32'
  })
  employees!: number;

  @ApiProperty({
    description: 'Company headquarters location',
    example: 'Cupertino, California',
    type: 'string'
  })
  headquarters!: string;
}

export class PaginatedStockResponse {
  @ApiProperty({
    description: 'Array of stock records for the current page',
    type: [StockResponseDto]
  })
  data!: StockResponseDto[];

  @ApiProperty({
    description: 'Total number of stock records available',
    type: 'number',
    minimum: 0,
    example: 500
  })
  total!: number;

  @ApiProperty({
    description: 'Current page number (1-based)',
    type: 'number',
    minimum: 1,
    example: 2
  })
  page!: number;

  @ApiProperty({
    description: 'Number of items per page',
    type: 'number',
    minimum: 1,
    maximum: 100,
    example: 20
  })
  limit!: number;
}

export class ScrapeByMarketCapDto {
  @ApiProperty({
    description: 'Minimum market capitalization in USD to include in scraping',
    type: 'number',
    minimum: 0,
    format: 'int64',
    example: 1000000000,
    default: 1000000000
  })
  minMarketCap!: number;

  @ApiProperty({
    description: 'Maximum market capitalization in USD to include in scraping',
    type: 'number',
    minimum: 0,
    format: 'int64',
    example: 5000000000000,
    default: 5000000000000
  })
  maxMarketCap!: number;
}

export class ScrapingStatusDto {
  @ApiProperty({
    description: 'Status of the scraping operation',
    enum: ['success', 'failure'],
    example: 'success'
  })
  status!: 'success' | 'failure';

  @ApiProperty({
    description: 'Detailed status message describing the scraping result',
    example: 'Stock data scraping completed successfully',
    type: 'string'
  })
  message!: string;

  @ApiProperty({
    description: 'When the scraping operation was performed',
    example: '2025-03-06T10:42:59Z',
    type: 'string',
    format: 'date-time'
  })
  timestamp!: string;

  @ApiProperty({
    description: 'Total number of stocks successfully scraped',
    example: 5000,
    type: 'number',
    minimum: 0,
    required: false
  })
  totalStocksScraped?: number;

  @ApiProperty({
    description: 'How long the scraping operation took',
    example: '2 hours 15 minutes',
    type: 'string',
    required: false
  })
  duration?: string;

  @ApiProperty({
    description: 'When the next scheduled scraping operation will occur',
    example: '2025-03-07T10:42:59Z',
    type: 'string',
    format: 'date-time',
    required: false
  })
  nextScrape?: string;
}
