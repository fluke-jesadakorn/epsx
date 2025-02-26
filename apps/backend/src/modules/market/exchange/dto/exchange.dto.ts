import { ApiProperty } from '@nestjs/swagger';
import { Exchange } from '@epsx/shared';

export class CreateExchangeDto implements Partial<Exchange> {
  @ApiProperty({
    description: 'Market code of the exchange',
    example: 'SET'
  })
  market_code!: string;

  @ApiProperty({
    description: 'Name of the exchange',
    example: 'Stock Exchange of Thailand'
  })
  exchange_name!: string;

  @ApiProperty({
    description: 'Country of the exchange',
    example: 'Thailand'
  })
  country?: string;

  @ApiProperty({
    description: 'Currency used in the exchange',
    example: 'THB'
  })
  currency?: string;

  @ApiProperty({
    description: 'Exchange website URL',
    example: 'https://www.set.or.th'
  })
  exchange_url?: string;

  @ApiProperty({
    description: 'Exchange timezone',
    example: 'Asia/Bangkok'
  })
  timezone?: string;
}

export class ExchangeResponseDto extends CreateExchangeDto {
  @ApiProperty({
    description: 'Unique identifier of the exchange',
    example: '507f1f77bcf86cd799439011'
  })
  _id!: string;

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

  @ApiProperty({
    description: 'MongoDB document version',
    example: 0
  })
  __v?: number;
}

export class PaginatedExchangeResponse {
  @ApiProperty({
    description: 'List of exchanges',
    type: [ExchangeResponseDto]
  })
  items!: ExchangeResponseDto[];

  @ApiProperty({
    description: 'Total number of exchanges',
    example: 100
  })
  total!: number;

  @ApiProperty({
    description: 'Current page number',
    example: 1
  })
  page!: number;

  @ApiProperty({
    description: 'Number of items per page',
    example: 10
  })
  limit!: number;
}
