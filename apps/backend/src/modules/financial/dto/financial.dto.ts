import { ApiProperty } from '@nestjs/swagger';

export interface ProcessingStatus {
  id: string;
  status: 'pending' | 'in_progress' | 'completed' | 'failed';
  message: string;
  progress?: number;
  totalSteps?: number;
  estimatedTimeRemaining?: string;
  error?: string;
}

export class ProcessingStatusDto implements ProcessingStatus {
  @ApiProperty({ 
    description: 'Unique identifier for the processing task',
    example: 'task_123456789'
  })
  id!: string;

  @ApiProperty({ 
    description: 'Current status of the processing task. Indicates whether the task is waiting to start (pending), currently running (in_progress), successfully completed (completed), or encountered an error (failed)',
    enum: ['pending', 'in_progress', 'completed', 'failed'],
    example: 'in_progress'
  })
  status!: 'pending' | 'in_progress' | 'completed' | 'failed';

  @ApiProperty({ 
    description: 'Descriptive message about the current state of the processing task',
    example: 'Processing financial data for Q4 2024'
  })
  message!: string;

  @ApiProperty({ 
    description: 'Current progress percentage of the task (0-100)',
    required: false,
    minimum: 0,
    maximum: 100,
    example: 75
  })
  progress?: number;

  @ApiProperty({ 
    description: 'Total number of steps in the processing task',
    required: false,
    minimum: 1,
    example: 10
  })
  totalSteps?: number;

  @ApiProperty({ 
    description: 'Estimated time remaining for task completion in human-readable format',
    required: false,
    example: '2 minutes'
  })
  estimatedTimeRemaining?: string;

  @ApiProperty({ 
    description: 'Detailed error message if the processing task failed',
    required: false,
    example: 'Failed to connect to financial data provider'
  })
  error?: string;
}

export class GetEPSGrowthRankingDto {
  @ApiProperty({ 
    description: 'Maximum number of items to return in the response',
    required: false,
    minimum: 1,
    maximum: 100,
    default: 10,
    example: 20
  })
  limit?: number;

  @ApiProperty({ 
    description: 'Number of items to skip for pagination',
    required: false,
    minimum: 0,
    default: 0,
    example: 20
  })
  skip?: number;

  @ApiProperty({ 
    description: 'Market code to filter stocks by (e.g., SET for Stock Exchange of Thailand)',
    required: false,
    example: 'SET'
  })
  market_code?: string;

  @ApiProperty({ 
    description: 'Field to sort results by. eps_growth sorts by EPS growth percentage, market_cap sorts by company market capitalization, volume sorts by trading volume',
    required: false,
    enum: ['eps_growth', 'market_cap', 'volume'],
    default: 'eps_growth'
  })
  sortBy?: string;

  @ApiProperty({
    description: 'Sort direction: ascending (asc) or descending (desc)',
    required: false,
    enum: ['asc', 'desc'],
    default: 'desc'
  })
  sortOrder?: 'asc' | 'desc';
}

import { EpsGrowthResponse } from '@epsx/shared';

export class PaginatedEpsGrowthResponse {
  @ApiProperty({ 
    description: 'Array of EPS growth records',
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
        quarter: { type: 'number', example: 4, minimum: 1, maximum: 4 }
      }
    }
  })
  data!: EpsGrowthResponse[];

  @ApiProperty({ 
    description: 'Total number of records available',
    example: 500
  })
  total!: number;

  @ApiProperty({ 
    description: 'Current page number (calculated from skip/limit)',
    example: 2
  })
  page!: number;

  @ApiProperty({ 
    description: 'Number of items per page',
    example: 20
  })
  limit!: number;
}
