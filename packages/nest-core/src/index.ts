// Re-export commonly used NestJS modules and decorators
export * from '@nestjs/common';
export * from '@nestjs/config';
export * from '@nestjs/microservices';
export * from '@nestjs/mongoose';
export * from '@nestjs/swagger';

// Export common interfaces and types
export interface HealthCheckResult {
  status: 'ok' | 'error';
  details?: Record<string, any>;
}

// Base configuration types
export interface AppConfig {
  port: number;
  environment: string;
}

export interface MongoConfig {
  uri: string;
  dbName: string;
}

// Common decorators
export * from './decorators';

// Common filters and interceptors
export * from './filters';
export * from './interceptors';

// Common services
export * from './services';

// Common utilities
export * from './utils';
