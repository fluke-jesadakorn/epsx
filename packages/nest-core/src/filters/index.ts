import {
  ExceptionFilter,
  Catch,
  ArgumentsHost,
  HttpException,
  HttpStatus,
  Logger,
} from '@nestjs/common';
import { Response, Request } from 'express';

@Catch()
export class GlobalExceptionFilter implements ExceptionFilter {
  private readonly logger = new Logger('GlobalExceptionFilter');

  catch(exception: unknown, host: ArgumentsHost) {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    const request = ctx.getRequest<Request>();

    let status = HttpStatus.INTERNAL_SERVER_ERROR;
    let message = 'Internal server error';
    let error = undefined;
    let errors = undefined;

    // Log detailed error information
    if (exception instanceof Error) {
      this.logger.error('Detailed error info:', {
        message: exception.message,
        stack: exception.stack,
        name: exception.name,
        cause: (exception as any).cause,
        metadata: (exception as any).metadata,
      });

      message = exception.message;
      error = exception.name;
    }

    // Log request details for debugging
    this.logger.debug('Request details:', {
      url: request.url,
      method: request.method,
      headers: request.headers,
      body: request.body,
      query: request.query,
    });

    if (exception instanceof HttpException) {
      status = exception.getStatus();
      const exceptionResponse = exception.getResponse();
      
      if (typeof exceptionResponse === 'string') {
        message = exceptionResponse;
      } else if (typeof exceptionResponse === 'object') {
        message = (exceptionResponse as any).message || message;
        error = (exceptionResponse as any).error;
        errors = (exceptionResponse as any).errors;
      }

      // Handle validation errors (BadRequestException)
      if (status === HttpStatus.BAD_REQUEST) {
        errors = (exceptionResponse as any).message || message;
      }
    }

    response.status(status).json({
      statusCode: status,
      timestamp: new Date().toISOString(),
      path: request.url,
      message,
      error,
      ...(errors && { errors }),
    });
  }
}

// Add more filters as needed
