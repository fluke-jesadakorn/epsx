import { ApiProperty } from '@nestjs/swagger';
import { DatabaseConnectionMetrics } from '../interfaces/database-config.interface';

export class DatabaseHealthCheckDto {
  @ApiProperty({
    description: 'Current health status of the database',
    enum: ['healthy', 'degraded', 'unhealthy'],
    example: 'healthy',
    default: 'healthy'
  })
  status!: 'healthy' | 'degraded' | 'unhealthy';

  @ApiProperty({
    description: 'Whether the application is currently connected to the database',
    example: true,
    type: 'boolean'
  })
  isConnected!: boolean;

  @ApiProperty({
    description: 'Timestamp of the last successful database heartbeat',
    type: 'string',
    format: 'date-time',
    example: '2024-03-10T02:30:00.000Z'
  })
  lastHeartbeat!: Date;

  @ApiProperty({
    description: 'Database query response time in milliseconds',
    type: 'number',
    minimum: 0,
    example: 50,
    required: false
  })
  responseTime?: number;

  @ApiProperty({
    description: 'Detailed database connection metrics',
    required: false,
    type: 'object',
    properties: {
      activeConnections: { type: 'number', example: 5, description: 'Number of active database connections' },
      maxConnections: { type: 'number', example: 20, description: 'Maximum allowed connections' },
      queriesPerSecond: { type: 'number', example: 100, description: 'Average queries executed per second' },
      memoryUsage: { type: 'number', example: 256, description: 'Memory usage in MB' }
    }
  })
  metrics?: DatabaseConnectionMetrics;
}

export class DatabaseConnectionEventDto {
  @ApiProperty({
    description: 'When the database connection event occurred',
    type: 'string',
    format: 'date-time',
    example: '2024-03-10T02:30:00.000Z'
  })
  timestamp!: Date;

  @ApiProperty({
    description: 'Type of database connection event',
    enum: ['connected', 'disconnected', 'reconnected', 'error'],
    example: 'connected',
    default: 'connected'
  })
  type!: 'connected' | 'disconnected' | 'reconnected' | 'error';

  @ApiProperty({
    description: 'Human-readable description of the event',
    example: 'Successfully connected to primary database',
    type: 'string'
  })
  message!: string;

  @ApiProperty({
    description: 'Error details if the event type is error',
    required: false,
    type: 'object',
    properties: {
      name: { type: 'string', example: 'ConnectionError' },
      message: { type: 'string', example: 'Connection timeout' },
      stack: { type: 'string', example: 'Error: Connection timeout\n    at Connection.connect...' }
    }
  })
  error?: Error;
}

export class HealthCheckResponseDto {
  @ApiProperty({
    description: 'Overall health status of the application',
    enum: ['healthy', 'degraded', 'unhealthy'],
    example: 'healthy',
    default: 'healthy'
  })
  status!: 'healthy' | 'degraded' | 'unhealthy';

  @ApiProperty({
    description: 'When the health check was performed',
    type: 'string',
    format: 'date-time',
    example: '2024-03-10T02:30:00.000Z'
  })
  timestamp!: string;

  @ApiProperty({
    description: 'Health status of individual services',
    type: 'object',
    properties: {
      database: {
        type: 'object',
        description: 'Database service health information',
        properties: {
          status: {
            type: 'string',
            enum: ['healthy', 'degraded', 'unhealthy'],
            description: 'Current health status of the database',
            example: 'healthy'
          },
          responseTime: {
            type: 'number',
            description: 'Database query response time in milliseconds',
            example: 50
          },
          isConnected: {
            type: 'boolean',
            description: 'Whether the application is connected to the database',
            example: true
          }
        }
      }
    }
  })
  services!: {
    database: {
      status: 'healthy' | 'degraded' | 'unhealthy';
      responseTime?: number;
      isConnected: boolean;
    };
  };
}

export class ReadinessCheckResponseDto {
  @ApiProperty({
    description: 'Application readiness status - whether it can handle requests',
    enum: ['ready', 'not_ready'],
    example: 'ready',
    default: 'ready'
  })
  status!: 'ready' | 'not_ready';

  @ApiProperty({
    description: 'When the readiness check was performed',
    type: 'string',
    format: 'date-time',
    example: '2024-03-10T02:30:00.000Z'
  })
  timestamp!: string;

  @ApiProperty({
    description: 'Additional information about the readiness state',
    required: false,
    example: 'All services initialized and ready to accept requests',
    type: 'string'
  })
  message?: string;
}

export class LivenessCheckResponseDto {
  @ApiProperty({
    description: 'Application liveness status - whether it is running',
    enum: ['alive'],
    example: 'alive'
  })
  status!: 'alive';

  @ApiProperty({
    description: 'When the liveness check was performed',
    type: 'string',
    format: 'date-time',
    example: '2024-03-10T02:30:00.000Z'
  })
  timestamp!: string;

  @ApiProperty({
    description: 'How long the application has been running in seconds',
    type: 'number',
    minimum: 0,
    example: 3600,
    format: 'float'
  })
  uptime!: number;
}
