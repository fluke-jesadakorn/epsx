// Services
export { ErrorHandlingService } from './services/error-handling.service';
export { DatabaseConfigService } from './services/database-config.service';
export { FirebaseAdminService } from './firebase-admin';
export { AuthService } from './auth';

// Guards and Decorators
export { FirebaseAuthGuard } from './guards/role.guard';
export { AuthorizationGuard } from './guards/roles.guard';
export { Roles } from './decorators/roles.decorator';
export { ROLES_KEY } from './decorators/roles.decorator';

// Controllers
export { HealthController } from './controllers/health.controller';

// DTOs
export {
  DatabaseHealthCheckDto,
  DatabaseConnectionEventDto,
  HealthCheckResponseDto,
  ReadinessCheckResponseDto,
  LivenessCheckResponseDto
} from './dto/health-check.dto';

// Interfaces and Types
export type {
  DatabaseConfig,
  DatabaseConnectionOptions,
  DatabaseConnectionEvent,
  DatabaseConnectionMetrics,
  DatabaseHealthCheck
} from './interfaces/database-config.interface';

// Enums
export { UserRole } from './types/roles.enum';

// Module
export { SharedModule } from './shared.module';

// Additional Types
export type { ErrorDetails } from './services/error-handling.service';

// Constants and configurations
export const HEALTH_CHECK_INTERVAL = 30000; // 30 seconds
export const MAX_CONNECTION_EVENTS = 100;

// Re-export commonly used types from external packages
export type { Connection } from 'mongoose';
export type { Request, Response } from 'express';
