import { Injectable, Logger, HttpException, HttpStatus } from '@nestjs/common';

export interface ErrorDetails {
  message: string;
  code?: string;
  timestamp?: string;
  path?: string;
  method?: string;
  userId?: string;
  metadata?: Record<string, any>;
}

@Injectable()
export class ErrorHandlingService {
  private readonly logger = new Logger(ErrorHandlingService.name);

  handleError(error: Error, details: ErrorDetails): never {
    const timestamp = new Date().toISOString();
    const errorLog = {
      ...details,
      timestamp,
      errorName: error.name,
      errorMessage: error.message,
      stackTrace: error.stack
    };

    // Log the error with all details
    this.logger.error(
      `Error occurred in ${details.path} (${details.method}):`,
      errorLog
    );

    // Determine if this is a known error type
    if (error instanceof HttpException) {
      throw error;
    }

    // Transform known error types
    if (error.name === 'ValidationError') {
      throw new HttpException({
        status: HttpStatus.BAD_REQUEST,
        error: 'Validation failed',
        message: error.message,
        timestamp,
        path: details.path
      }, HttpStatus.BAD_REQUEST);
    }

    if (error.name === 'NotFoundError') {
      throw new HttpException({
        status: HttpStatus.NOT_FOUND,
        error: 'Resource not found',
        message: error.message,
        timestamp,
        path: details.path
      }, HttpStatus.NOT_FOUND);
    }

    // Default to internal server error for unknown error types
    throw new HttpException({
      status: HttpStatus.INTERNAL_SERVER_ERROR,
      error: 'Internal server error',
      message: 'An unexpected error occurred',
      timestamp,
      path: details.path
    }, HttpStatus.INTERNAL_SERVER_ERROR);
  }

  logWarning(message: string, details: ErrorDetails): void {
    const warningLog = {
      ...details,
      timestamp: new Date().toISOString(),
      message
    };

    this.logger.warn(message, warningLog);
  }

  logInfo(message: string, details: Partial<ErrorDetails>): void {
    const infoLog = {
      ...details,
      timestamp: new Date().toISOString(),
      message
    };

    this.logger.log(message, infoLog);
  }

  handleOperationalError(error: Error, operation: string, details: ErrorDetails): never {
    const errorLog = {
      ...details,
      operation,
      timestamp: new Date().toISOString(),
      errorName: error.name,
      errorMessage: error.message
    };

    this.logger.error(
      `Operational error in ${operation}:`,
      errorLog
    );

    throw new HttpException({
      status: HttpStatus.SERVICE_UNAVAILABLE,
      error: 'Operation failed',
      message: `Failed to perform ${operation}`,
      timestamp: errorLog.timestamp,
      path: details.path
    }, HttpStatus.SERVICE_UNAVAILABLE);
  }

  handleDatabaseError(error: Error, operation: string, details: ErrorDetails): never {
    const errorLog = {
      ...details,
      operation,
      timestamp: new Date().toISOString(),
      errorName: error.name,
      errorMessage: error.message
    };

    this.logger.error(
      `Database error in ${operation}:`,
      errorLog
    );

    throw new HttpException({
      status: HttpStatus.INTERNAL_SERVER_ERROR,
      error: 'Database operation failed',
      message: 'Failed to process database operation',
      timestamp: errorLog.timestamp,
      path: details.path
    }, HttpStatus.INTERNAL_SERVER_ERROR);
  }

  handleExternalServiceError(error: Error, service: string, details: ErrorDetails): never {
    const errorLog = {
      ...details,
      service,
      timestamp: new Date().toISOString(),
      errorName: error.name,
      errorMessage: error.message
    };

    this.logger.error(
      `External service error with ${service}:`,
      errorLog
    );

    throw new HttpException({
      status: HttpStatus.BAD_GATEWAY,
      error: 'External service error',
      message: `Failed to communicate with ${service}`,
      timestamp: errorLog.timestamp,
      path: details.path
    }, HttpStatus.BAD_GATEWAY);
  }
}
