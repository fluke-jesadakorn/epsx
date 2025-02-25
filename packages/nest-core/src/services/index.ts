import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { MongoConfig } from '../index';

@Injectable()
export class BaseConfigService {
  constructor(protected configService: ConfigService) {}

  get<T>(key: string): T | undefined {
    return this.configService.get<T>(key);
  }

  getMongoConfig(): MongoConfig {
    const uri = this.get<string>('MONGODB_URI') ?? 'mongodb://localhost:27017';
    const dbName = this.get<string>('MONGODB_NAME') ?? 'epsx';
    return {
      uri,
      dbName,
    };
  }

  getPort(): number {
    return this.get<number>('PORT') || 3000;
  }

  getEnvironment(): string {
    return this.get<string>('NODE_ENV') || 'development';
  }
}

@Injectable()
export class BaseHealthService {
  async check() {
    return {
      status: 'ok',
      timestamp: new Date().toISOString(),
    };
  }
}

// Add more base services as needed
