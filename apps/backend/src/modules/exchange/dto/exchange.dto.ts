import { ApiProperty } from '@nestjs/swagger';
import { IExchange } from '@epsx/shared';

export class CreateExchangeDto implements Partial<IExchange> {
  @ApiProperty({
    description: 'Unique market code identifier for the exchange (e.g., SET, NYSE, NASDAQ)',
    example: 'SET',
    minLength: 2,
    maxLength: 10,
    pattern: '^[A-Z]+$',
    type: 'string'
  })
  market_code!: string;

  @ApiProperty({
    description: 'Official name of the stock exchange',
    example: 'Stock Exchange of Thailand',
    minLength: 3,
    maxLength: 100,
    type: 'string'
  })
  exchange_name!: string;

  @ApiProperty({
    description: 'Country where the exchange is located',
    example: 'Thailand',
    minLength: 2,
    maxLength: 50,
    type: 'string',
    required: false
  })
  country?: string;

  @ApiProperty({
    description: 'ISO 4217 currency code used for trading (e.g., THB, USD, EUR)',
    example: 'THB',
    minLength: 3,
    maxLength: 3,
    pattern: '^[A-Z]{3}$',
    type: 'string',
    required: false
  })
  currency?: string;

  @ApiProperty({
    description: 'Official website URL of the stock exchange',
    example: 'https://www.set.or.th',
    pattern: '^https?://[\\w\\-]+(\\.[\\w\\-]+)+[/#?]?.*$',
    type: 'string',
    format: 'uri',
    required: false
  })
  exchange_url?: string;

  @ApiProperty({
    description: 'IANA timezone identifier for the exchange (e.g., Asia/Bangkok, America/New_York)',
    example: 'Asia/Bangkok',
    pattern: '^[A-Za-z_]+/[A-Za-z_]+$',
    type: 'string',
    required: false
  })
  timezone?: string;
}

export class ExchangeResponseDto extends CreateExchangeDto {
  @ApiProperty({
    description: 'MongoDB ObjectId unique identifier of the exchange',
    example: '507f1f77bcf86cd799439011',
    type: 'string',
    format: 'mongo-id',
    pattern: '^[0-9a-fA-F]{24}$'
  })
  _id!: string;

  @ApiProperty({
    description: 'Timestamp when the exchange record was created',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time'
  })
  createdAt?: Date;

  @ApiProperty({
    description: 'Timestamp when the exchange record was last modified',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time'
  })
  updatedAt?: Date;

  @ApiProperty({
    description: 'MongoDB document version for optimistic concurrency control',
    example: 0,
    type: 'number',
    minimum: 0,
    default: 0
  })
  __v?: number;
}

export class ExchangeScrapeResponseDto {
  @ApiProperty({
    description: 'Status of the scraping operation',
    example: 'success',
    type: 'string',
    enum: ['success', 'error']
  })
  status!: string;

  @ApiProperty({
    description: 'Number of new records added',
    example: 5,
    type: 'number',
    minimum: 0
  })
  addNewRecord!: number;
}

export class PaginatedExchangeResponse {
  @ApiProperty({
    description: 'Array of exchange records for the current page',
    type: 'array',
    items: {
      type: 'object',
      $ref: '#/components/schemas/ExchangeResponseDto'
    },
    example: [{
      _id: '507f1f77bcf86cd799439011',
      market_code: 'SET',
      exchange_name: 'Stock Exchange of Thailand',
      country: 'Thailand',
      currency: 'THB',
      exchange_url: 'https://www.set.or.th',
      timezone: 'Asia/Bangkok',
      createdAt: '2024-02-26T08:30:00.000Z',
      updatedAt: '2024-02-26T08:30:00.000Z',
      __v: 0
    }]
  })
  items!: ExchangeResponseDto[];

  @ApiProperty({
    description: 'Total number of exchange records in the database',
    example: 100,
    type: 'number',
    minimum: 0
  })
  total!: number;

  @ApiProperty({
    description: 'Current page number (1-based)',
    example: 1,
    type: 'number',
    minimum: 1,
    default: 1
  })
  page!: number;

  @ApiProperty({
    description: 'Maximum number of items returned per page',
    example: 10,
    type: 'number',
    minimum: 1,
    maximum: 100,
    default: 10
  })
  limit!: number;
}
