import { Injectable, OnModuleInit, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { MongooseModuleOptions, MongooseOptionsFactory } from '@nestjs/mongoose';
import { Connection, Model } from 'mongoose';
import { InjectModel } from '@nestjs/mongoose';
import { 
  DatabaseConfig, 
  DatabaseConnectionEvent, 
  DatabaseHealthCheck,
  DatabaseConnectionMetrics 
} from '../interfaces/database-config.interface';
import {
  HealthCheckLog,
  HealthCheckLogDocument,
  ConnectionEvent,
  ConnectionEventDocument
} from '../schemas/health-check.schema';

@Injectable()
export class DatabaseConfigService implements MongooseOptionsFactory, OnModuleInit {
  private readonly logger = new Logger(DatabaseConfigService.name);
  private connection: Connection | null = null;
  private lastHealthCheck: DatabaseHealthCheck | null = null;

  private healthCheckModel?: Model<HealthCheckLogDocument>;
  private connectionEventModel?: Model<ConnectionEventDocument>;

  constructor(
    private configService: ConfigService,
  ) {}

  setModels(
    healthCheckModel: Model<HealthCheckLogDocument>,
    connectionEventModel: Model<ConnectionEventDocument>
  ) {
    this.healthCheckModel = healthCheckModel;
    this.connectionEventModel = connectionEventModel;
  }

  onModuleInit() {
    // Start health check monitoring
    setInterval(() => {
      void this.performHealthCheck();
    }, 30000); // Check every 30 seconds
  }

  createMongooseOptions(): MongooseModuleOptions {
    const uri = this.getRequiredConfig('MONGODB_URI');

    return {
      uri,
      retryAttempts: 3,
      retryDelay: 1000,
      connectionFactory: (connection) => {
        this.connection = connection;

        connection.on('connected', () => {
          void this.logConnectionEvent('connected', 'MongoDB connection established');
        });

        connection.on('disconnected', () => {
          void this.logConnectionEvent('disconnected', 'MongoDB disconnected');
        });

        connection.on('reconnected', () => {
          void this.logConnectionEvent('reconnected', 'MongoDB reconnected');
        });

        connection.on('error', (error: Error) => {
          void this.logConnectionEvent('error', 'MongoDB connection error', error);
        });

        if (process.env.NODE_ENV !== 'production') {
          connection.set('debug', true);
          connection.set('autoIndex', true);
        }

        return connection;
      },
      autoIndex: process.env.NODE_ENV !== 'production',
      autoCreate: process.env.NODE_ENV !== 'production',
    };
  }

  getConnectionString(): string {
    return this.getRequiredConfig('MONGODB_URI');
  }

  getMongooseOptions(): MongooseModuleOptions {
    return this.createMongooseOptions();
  }

  async getHealthCheck(): Promise<DatabaseHealthCheck> {
    try {
      if (!this.lastHealthCheck) {
        await this.performHealthCheck();
      }
      return this.lastHealthCheck ?? this.createUnhealthyStatus();
    } catch (error) {
      this.logger.error('Failed to get health check:', error);
      return this.createUnhealthyStatus();
    }
  }

  async getConnectionEvents(): Promise<DatabaseConnectionEvent[]> {
    try {
      if (!this.connectionEventModel) {
        return [];
      }
      const events = await this.connectionEventModel.find()
        .sort({ timestamp: -1 })
        .limit(100)
        .exec();

      return events.map(event => ({
        type: event.type,
        timestamp: event.timestamp,
        message: event.message,
        error: event.error
      }));
    } catch (error) {
      this.logger.error('Failed to get connection events:', error);
      return [];
    }
  }

  private getRequiredConfig(key: keyof DatabaseConfig): string {
    const value = this.configService.get<string>(key);
    if (!value) {
      throw new Error(`Required configuration ${key} is missing`);
    }
    return value;
  }

  private createUnhealthyStatus(): DatabaseHealthCheck {
    return {
      isConnected: false,
      status: 'unhealthy',
      lastHeartbeat: new Date(),
      metrics: this.getConnectionMetrics()
    };
  }

  private async performHealthCheck(): Promise<void> {
    if (!this.connection) {
      this.lastHealthCheck = this.createUnhealthyStatus();
      return;
    }

    try {
      // Ensure we have a database instance
      if (!this.connection.db) {
        throw new Error('Database instance not available');
      }

      const startTime = Date.now();
      // Ping the database
      await this.connection.db.admin().ping();
      const responseTime = Date.now() - startTime;

      const metrics = this.getConnectionMetrics();
      
      this.lastHealthCheck = {
        isConnected: this.connection.readyState === 1,
        lastHeartbeat: new Date(),
        responseTime,
        status: this.determineHealthStatus(responseTime, metrics),
        metrics
      };

      // Save health check to database
      if (this.healthCheckModel) {
        await this.healthCheckModel.create({
          status: this.lastHealthCheck.status,
          timestamp: this.lastHealthCheck.lastHeartbeat,
          responseTime: this.lastHealthCheck.responseTime,
          isConnected: this.lastHealthCheck.isConnected,
          metrics: this.lastHealthCheck.metrics
        });
      }
    } catch (error) {
      this.logger.error('Health check failed:', error);
      this.lastHealthCheck = this.createUnhealthyStatus();
    }
  }

  private determineHealthStatus(
    responseTime: number,
    metrics: DatabaseConnectionMetrics
  ): 'healthy' | 'degraded' | 'unhealthy' {
    if (!this.connection || this.connection.readyState !== 1) {
      return 'unhealthy';
    }

    if (responseTime > 1000 || metrics.operationsQueued > 100) {
      return 'degraded';
    }

    return 'healthy';
  }

  private getConnectionMetrics(): DatabaseConnectionMetrics {
    if (!this.connection) {
      return {
        connectionsActive: 0,
        connectionsAvailable: 0,
        connectionsCreated: 0,
        connectionsReused: 0,
        operationsQueued: 0
      };
    }

    return {
      connectionsActive: this.connection.readyState,
      connectionsAvailable: 0, // Mongoose doesn't expose this directly
      connectionsCreated: 0, // Mongoose doesn't expose this directly
      connectionsReused: 0, // Mongoose doesn't expose this directly
      operationsQueued: 0 // Mongoose doesn't expose this directly
    };
  }

  private async logConnectionEvent(
    type: DatabaseConnectionEvent['type'],
    message: string,
    error?: Error
  ): Promise<void> {
    try {
      // Create event document
      if (!this.connectionEventModel) return;
      
      const event = await this.connectionEventModel.create({
        type,
        timestamp: new Date(),
        message,
        ...(error && { error: {
          name: error.name,
          message: error.message,
          stack: error.stack
        }})
      });

      // Create health check log if it's a connection status change
      if (['connected', 'disconnected', 'reconnected'].includes(type)) {
        if (this.healthCheckModel) {
          await this.healthCheckModel.create({
            status: type === 'connected' ? 'healthy' : 'unhealthy',
            timestamp: new Date(),
            isConnected: type === 'connected',
            metrics: this.getConnectionMetrics()
          });
        }
      }

      // Cleanup old events (keep last 100)
      const oldEvents = await this.connectionEventModel
        .find()
        .sort({ timestamp: -1 })
        .skip(100)
        .exec();

      if (oldEvents.length > 0) {
        const oldestDate = oldEvents[oldEvents.length - 1].timestamp;
        await this.connectionEventModel.deleteMany({
          timestamp: { $lt: oldestDate }
        }).exec();
      }
    } catch (error) {
      this.logger.error('Failed to log connection event:', error);
    }
  }
}
