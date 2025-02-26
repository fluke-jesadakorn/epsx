import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';
import { ApiProperty } from '@nestjs/swagger';

@Schema({ timestamps: true })
export class EPSGrowthProcessing extends Document {
  @ApiProperty({
    description: 'Stock symbol being processed',
    example: 'AAPL',
    type: 'string'
  })
  @Prop({ required: true })
  symbol!: string;

  @ApiProperty({
    description: 'Market code where the stock is listed',
    example: 'SET',
    type: 'string'
  })
  @Prop({ required: true })
  market_code!: string;

  @ApiProperty({
    description: 'Current status of the processing job',
    enum: ['pending', 'processing', 'completed', 'failed'],
    example: 'processing',
    type: 'string'
  })
  @Prop({ required: true })
  status!: 'pending' | 'processing' | 'completed' | 'failed';

  @ApiProperty({
    description: 'Error message if processing failed',
    example: 'Failed to fetch data from API',
    type: 'string',
    required: false
  })
  @Prop()
  error?: string;

  @ApiProperty({
    description: 'Number of stocks processed so far',
    example: 50,
    type: 'number',
    minimum: 0
  })
  @Prop({ required: true, type: Number })
  processedStocks: number = 0;

  @ApiProperty({
    description: 'Total number of stocks to process',
    example: 100,
    type: 'number',
    minimum: 0
  })
  @Prop({ required: true, type: Number })
  totalStocks: number = 0;

  @ApiProperty({
    description: 'Last stock symbol that was processed',
    example: 'GOOGL',
    type: 'string',
    required: false
  })
  @Prop()
  lastProcessedSymbol?: string;

  @ApiProperty({
    description: 'Whether the processing job is complete',
    example: false,
    type: 'boolean'
  })
  @Prop({ type: Boolean, default: false })
  isCompleted: boolean = false;

  @ApiProperty({
    description: 'When the processing job was requested',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false
  })
  @Prop({ type: Date })
  requested_at?: Date;

  @ApiProperty({
    description: 'When the processing job was completed',
    example: '2024-02-26T08:35:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false
  })
  @Prop({ type: Date })
  completed_at?: Date;
}

export const EPSGrowthProcessingSchema = SchemaFactory.createForClass(EPSGrowthProcessing);

@Schema({ timestamps: true })
export class EPSGrowthBatch extends Document {
  @ApiProperty({
    description: 'Market code for this batch',
    example: 'SET',
    type: 'string'
  })
  @Prop({ required: true })
  market_code!: string;

  @ApiProperty({
    description: 'Array of stock symbols in this batch',
    example: ['AAPL', 'GOOGL', 'MSFT'],
    type: 'array',
    items: {
      type: 'string'
    }
  })
  @Prop({ required: true, type: [String] })
  symbols!: string[];

  @ApiProperty({
    description: 'Current status of the batch processing',
    enum: ['pending', 'processing', 'completed', 'failed'],
    example: 'processing',
    type: 'string'
  })
  @Prop({ required: true })
  status!: 'pending' | 'processing' | 'completed' | 'failed';

  @ApiProperty({
    description: 'Error message if batch processing failed',
    example: 'API rate limit exceeded',
    type: 'string',
    required: false
  })
  @Prop()
  error?: string;

  @ApiProperty({
    description: 'Whether the batch has been processed',
    example: false,
    type: 'boolean'
  })
  @Prop({ type: Boolean, default: false })
  isProcessed: boolean = false;

  @ApiProperty({
    description: 'Processing results for each symbol',
    type: 'array',
    items: {
      type: 'object'
    }
  })
  @Prop({ type: Array, default: [] })
  results: any[] = [];

  @ApiProperty({
    description: 'When the batch was requested for processing',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time'
  })
  @Prop({ required: true, type: Date })
  requested_at!: Date;

  @ApiProperty({
    description: 'When the batch processing was completed',
    example: '2024-02-26T08:35:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false
  })
  @Prop({ type: Date })
  completed_at?: Date;

  @ApiProperty({
    description: 'Number of symbols processed in this batch',
    example: 25,
    type: 'number',
    minimum: 0
  })
  @Prop({ type: Number, default: 0 })
  processed_count: number = 0;

  @ApiProperty({
    description: 'Total number of symbols in this batch',
    example: 50,
    type: 'number',
    minimum: 0
  })
  @Prop({ type: Number, default: 0 })
  total_count: number = 0;
}

export const EPSGrowthBatchSchema = SchemaFactory.createForClass(EPSGrowthBatch);
