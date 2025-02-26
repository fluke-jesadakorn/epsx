import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';
import { ApiProperty } from '@nestjs/swagger';

@Schema({ timestamps: true })
export class EpsGrowth extends Document {
  @ApiProperty({
    description: 'Stock symbol/ticker',
    example: 'AAPL',
    type: 'string'
  })
  @Prop({ required: true })
  symbol!: string;

  @ApiProperty({
    description: 'Company name',
    example: 'Apple Inc.',
    type: 'string'
  })
  @Prop({ required: true })
  company_name!: string;

  @ApiProperty({
    description: 'Market code where the stock is listed',
    example: 'SET',
    type: 'string'
  })
  @Prop({ required: true })
  market_code!: string;

  @ApiProperty({
    description: 'Current period diluted Earnings Per Share',
    example: 6.42,
    type: 'number',
    format: 'float'
  })
  @Prop({ required: true })
  eps_diluted!: number;

  @ApiProperty({
    description: 'Previous period diluted Earnings Per Share',
    example: 5.55,
    type: 'number',
    format: 'float'
  })
  @Prop({ required: true })
  previous_eps_diluted!: number;

  @ApiProperty({
    description: 'EPS growth percentage compared to previous period',
    example: 15.7,
    type: 'number',
    format: 'float'
  })
  @Prop({ required: true })
  eps_growth!: number;

  @ApiProperty({
    description: 'Financial report publication date',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time'
  })
  @Prop({ required: true, type: Date })
  report_date!: Date;

  @ApiProperty({
    description: 'Fiscal year',
    example: 2024,
    type: 'number',
    minimum: 1900
  })
  @Prop({ required: true })
  year!: number;

  @ApiProperty({
    description: 'Fiscal quarter (1-4)',
    example: 4,
    type: 'number',
    minimum: 1,
    maximum: 4
  })
  @Prop({ required: true })
  quarter!: number;
}

export const EpsGrowthSchema = SchemaFactory.createForClass(EpsGrowth);
