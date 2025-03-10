import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import type { Document } from 'mongoose';
import { ApiProperty } from '@nestjs/swagger';

export interface IEPSGrowthProcessing {
  symbol: string;
  market_code: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  error?: string;
  processedStocks: number;
  totalStocks: number;
  lastProcessedSymbol?: string;
  isCompleted: boolean;
  requested_at?: Date;
  completed_at?: Date;
}

export interface IEPSGrowthBatch {
  market_code: string;
  symbols: string[];
  status: 'pending' | 'processing' | 'completed' | 'failed';
  error?: string;
  isProcessed: boolean;
  results: any[];
  requested_at: Date;
  completed_at?: Date;
  processed_count: number;
  total_count: number;
}

export type EPSGrowthProcessingDocument = Document & IEPSGrowthProcessing;

@Schema({ timestamps: true })
export class EPSGrowthProcessing implements IEPSGrowthProcessing {
  @ApiProperty({
    description: 'Stock symbol being processed. Must be a valid stock market ticker symbol.',
    example: 'AAPL',
    type: 'string',
    minLength: 1,
    maxLength: 5,
    pattern: '^[A-Z]{1,5}$'
  })
  @Prop({ type: String, required: true })
  symbol!: string;

  @ApiProperty({
    description: 'Market code where the stock is listed (e.g., SET for Stock Exchange of Thailand, NYSE for New York Stock Exchange)',
    example: 'SET',
    type: 'string'
  })
  @Prop({ type: String, required: true })
  market_code!: string;

  @ApiProperty({
    description: 'Current processing status. Indicates whether the task is waiting to start (pending), currently being processed (processing), finished successfully (completed), or encountered an error (failed)',
    enum: ['pending', 'processing', 'completed', 'failed'],
    example: 'processing',
    type: 'string',
    default: 'pending'
  })
  @Prop({ type: String, enum: ['pending', 'processing', 'completed', 'failed'], required: true })
  status!: 'pending' | 'processing' | 'completed' | 'failed';

  @ApiProperty({
    description: 'Detailed error message if processing failed. Null if processing is successful.',
    example: 'Failed to fetch data from financial API: Rate limit exceeded',
    type: 'string',
    required: false,
    nullable: true
  })
  @Prop({ type: String })
  error?: string;

  @ApiProperty({
    description: 'Number of stocks that have been processed in the current batch',
    example: 50,
    type: 'number',
    minimum: 0,
    default: 0
  })
  @Prop({ required: true, type: Number })
  processedStocks: number = 0;

  @ApiProperty({
    description: 'Total number of stocks that need to be processed in this batch',
    example: 100,
    type: 'number',
    minimum: 0,
    default: 0
  })
  @Prop({ required: true, type: Number })
  totalStocks: number = 0;

  @ApiProperty({
    description: 'The most recently processed stock symbol. Useful for tracking progress and resuming interrupted operations.',
    example: 'GOOGL',
    type: 'string',
    required: false,
    nullable: true
  })
  @Prop({ type: String })
  lastProcessedSymbol?: string;

  @ApiProperty({
    description: 'Indicates whether all stocks in this batch have been processed',
    example: false,
    type: 'boolean',
    default: false
  })
  @Prop({ type: Boolean, default: false })
  isCompleted: boolean = false;

  @ApiProperty({
    description: 'Timestamp when the processing job was initially requested',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false,
    nullable: true
  })
  @Prop({ type: Date })
  requested_at?: Date;

  @ApiProperty({
    description: 'Timestamp when the processing job was fully completed. Null if still in progress.',
    example: '2024-02-26T08:35:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false,
    nullable: true
  })
  @Prop({ type: Date })
  completed_at?: Date;
}

export const EPSGrowthProcessingSchema = SchemaFactory.createForClass(EPSGrowthProcessing);

export type EPSGrowthBatchDocument = Document & IEPSGrowthBatch;

@Schema({ timestamps: true })
export class EPSGrowthBatch implements IEPSGrowthBatch {
  @ApiProperty({
    description: 'Market code identifying the stock exchange for this batch of symbols',
    example: 'SET',
    type: 'string'
  })
  @Prop({ type: String, required: true })
  market_code!: string;

  @ApiProperty({
    description: 'Array of stock symbols to process in this batch. Each symbol must be a valid stock market ticker.',
    example: ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'META'],
    type: 'array',
    items: {
      type: 'string',
      pattern: '^[A-Z]{1,5}$',
      description: 'Stock symbol in uppercase, 1-5 characters'
    },
    minItems: 1,
    uniqueItems: true
  })
  @Prop({ required: true, type: [String] })
  symbols!: string[];

  @ApiProperty({
    description: 'Current status of the batch processing job',
    enum: ['pending', 'processing', 'completed', 'failed'],
    example: 'processing',
    type: 'string',
    default: 'pending'
  })
  @Prop({ type: String, enum: ['pending', 'processing', 'completed', 'failed'], required: true })
  status!: 'pending' | 'processing' | 'completed' | 'failed';

  @ApiProperty({
    description: 'Detailed error message if batch processing failed. Null if processing is successful.',
    example: 'API rate limit exceeded - waiting for reset',
    type: 'string',
    required: false,
    nullable: true
  })
  @Prop({ type: String })
  error?: string;

  @ApiProperty({
    description: 'Indicates whether this batch has been fully processed',
    example: false,
    type: 'boolean',
    default: false
  })
  @Prop({ type: Boolean, default: false })
  isProcessed: boolean = false;

  @ApiProperty({
    description: 'Array containing the processing results for each symbol',
    type: 'array',
    items: {
      type: 'object',
      properties: {
        symbol: {
          type: 'string',
          example: 'AAPL',
          description: 'Stock symbol that was processed'
        },
        eps_growth: {
          type: 'number',
          example: 15.7,
          description: 'Calculated EPS growth percentage'
        },
        eps_current: {
          type: 'number',
          example: 6.42,
          description: 'Current period EPS value'
        },
        eps_previous: {
          type: 'number',
          example: 5.55,
          description: 'Previous period EPS value'
        },
        processed_at: {
          type: 'string',
          format: 'date-time',
          example: '2024-02-26T08:32:00.000Z',
          description: 'Timestamp when this symbol was processed'
        },
        status: {
          type: 'string',
          enum: ['success', 'failed'],
          example: 'success',
          description: 'Processing status for this specific symbol'
        },
        error: {
          type: 'string',
          example: null,
          description: 'Error message if processing failed for this symbol'
        }
      }
    },
    default: []
  })
  @Prop({ type: Array, default: [] })
  results: any[] = [];

  @ApiProperty({
    description: 'Timestamp when the batch was initially submitted for processing',
    example: '2024-02-26T08:30:00.000Z',
    type: 'string',
    format: 'date-time'
  })
  @Prop({ required: true, type: Date })
  requested_at!: Date;

  @ApiProperty({
    description: 'Timestamp when batch processing was completed. Null if still in progress.',
    example: '2024-02-26T08:35:00.000Z',
    type: 'string',
    format: 'date-time',
    required: false,
    nullable: true
  })
  @Prop({ type: Date })
  completed_at?: Date;

  @ApiProperty({
    description: 'Number of symbols successfully processed in this batch',
    example: 25,
    type: 'number',
    minimum: 0,
    default: 0
  })
  @Prop({ type: Number, default: 0 })
  processed_count: number = 0;

  @ApiProperty({
    description: 'Total number of symbols in this batch that need to be processed',
    example: 50,
    type: 'number',
    minimum: 0,
    default: 0
  })
  @Prop({ type: Number, default: 0 })
  total_count: number = 0;
}

export const EPSGrowthBatchSchema = SchemaFactory.createForClass(EPSGrowthBatch);
