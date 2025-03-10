import { Controller, Get, HttpException } from "@nestjs/common";
import { ApiTags, ApiOperation, ApiResponse } from "@nestjs/swagger";
import { DatabaseConfigService } from "../services/database-config.service";
import { ErrorHandlingService } from "../services/error-handling.service";
import { Roles } from "../decorators/roles.decorator";
import { UserRole } from "../types/roles.enum";
import {
  DatabaseHealthCheckDto,
  DatabaseConnectionEventDto,
  HealthCheckResponseDto,
  ReadinessCheckResponseDto,
  LivenessCheckResponseDto,
} from "../dto/health-check.dto";
import {
  HEALTH_CHECK_DEFAULTS,
  HEALTH_CHECK_PATHS,
  HEALTH_CHECK_TAGS,
  HealthCheckHttpStatus,
} from "./index";

@ApiTags(HEALTH_CHECK_TAGS.HEALTH)
@Controller(HEALTH_CHECK_PATHS.BASE.slice(1)) // Remove leading slash
export class HealthController {
  constructor(
    private readonly databaseConfigService: DatabaseConfigService,
    private readonly errorHandlingService: ErrorHandlingService
  ) {}

  @Get()
  @ApiOperation({ summary: "Get overall system health status" })
  @ApiResponse({
    status: HealthCheckHttpStatus.OK,
    description: "System health check results",
    type: HealthCheckResponseDto,
  })
  @ApiResponse({
    status: HealthCheckHttpStatus.SERVICE_UNAVAILABLE,
    description: "System is unhealthy",
  })
  async getHealth(): Promise<HealthCheckResponseDto> {
    try {
      const dbHealth = await this.databaseConfigService.getHealthCheck();

      const response: HealthCheckResponseDto = {
        status: dbHealth.status,
        timestamp: new Date().toISOString(),
        services: {
          database: {
            status: dbHealth.status,
            responseTime: dbHealth.responseTime ?? 0,
            isConnected: dbHealth.isConnected,
          },
        },
      };

      if (dbHealth.status === "unhealthy") {
        throw new HttpException(
          response,
          HealthCheckHttpStatus.SERVICE_UNAVAILABLE
        );
      }

      return response;
    } catch (error: unknown) {
      if (error instanceof HttpException) {
        throw error;
      }
      throw this.errorHandlingService.handleError(
        error instanceof Error ? error : new Error(String(error)),
        {
          path: HEALTH_CHECK_PATHS.BASE,
          method: "GET",
          message: "Failed to get system health status",
        }
      );
    }
  }

  @Get("database/details")
  @Roles(UserRole.ADMINISTRATOR)
  @ApiOperation({ summary: "Get detailed database health information" })
  @ApiResponse({
    status: HealthCheckHttpStatus.OK,
    description: "Detailed database health information",
    type: DatabaseHealthCheckDto,
  })
  @ApiResponse({
    status: HealthCheckHttpStatus.UNAUTHORIZED,
    description: "Unauthorized access",
  })
  async getDatabaseHealth(): Promise<DatabaseHealthCheckDto> {
    try {
      const healthCheck = await this.databaseConfigService.getHealthCheck();
      return {
        status: healthCheck.status,
        isConnected: healthCheck.isConnected,
        lastHeartbeat: healthCheck.lastHeartbeat ?? new Date(),
        responseTime: healthCheck.responseTime,
        metrics: healthCheck.metrics,
      };
    } catch (error: unknown) {
      throw this.errorHandlingService.handleError(
        error instanceof Error ? error : new Error(String(error)),
        {
          path: HEALTH_CHECK_PATHS.DATABASE_DETAILS,
          method: "GET",
          message: "Failed to get database health details",
        }
      );
    }
  }

  @Get("database/events")
  @Roles(UserRole.ADMINISTRATOR)
  @ApiOperation({ summary: "Get database connection events" })
  @ApiResponse({
    status: HealthCheckHttpStatus.OK,
    description: "List of database connection events",
    type: [DatabaseConnectionEventDto],
  })
  @ApiResponse({
    status: HealthCheckHttpStatus.UNAUTHORIZED,
    description: "Unauthorized access",
  })
  async getDatabaseEvents(): Promise<DatabaseConnectionEventDto[]> {
    try {
      return await this.databaseConfigService.getConnectionEvents();
    } catch (error: unknown) {
      throw this.errorHandlingService.handleError(
        error instanceof Error ? error : new Error(String(error)),
        {
          path: HEALTH_CHECK_PATHS.DATABASE_EVENTS,
          method: "GET",
          message: "Failed to get database connection events",
        }
      );
    }
  }

  @Get("readiness")
  @ApiOperation({
    summary: "Check if the application is ready to receive traffic",
  })
  @ApiResponse({
    status: HealthCheckHttpStatus.OK,
    description: "Application is ready",
    type: ReadinessCheckResponseDto,
  })
  @ApiResponse({
    status: HealthCheckHttpStatus.SERVICE_UNAVAILABLE,
    description: "Application is not ready",
    type: ReadinessCheckResponseDto,
  })
  async getReadiness(): Promise<ReadinessCheckResponseDto> {
    try {
      const dbHealth = await this.databaseConfigService.getHealthCheck();
      const response: ReadinessCheckResponseDto = {
        status: dbHealth.status === "unhealthy" ? "not_ready" : "ready",
        timestamp: new Date().toISOString(),
      };

      if (dbHealth.status === "unhealthy") {
        response.message = "Database connection is not healthy";
        throw new HttpException(
          response,
          HealthCheckHttpStatus.SERVICE_UNAVAILABLE
        );
      }

      return response;
    } catch (error: unknown) {
      if (error instanceof HttpException) {
        throw error;
      }
      throw this.errorHandlingService.handleError(
        error instanceof Error ? error : new Error(String(error)),
        {
          path: HEALTH_CHECK_PATHS.READINESS,
          method: "GET",
          message: "Failed to check application readiness",
        }
      );
    }
  }

  @Get("liveness")
  @ApiOperation({ summary: "Check if the application is alive" })
  @ApiResponse({
    status: HealthCheckHttpStatus.OK,
    description: "Application is alive",
    type: LivenessCheckResponseDto,
  })
  getLiveness(): LivenessCheckResponseDto {
    return {
      status: "alive",
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
    };
  }
}
