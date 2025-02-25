import { NestFactory } from "@nestjs/core";
import { Logger, Catch, ArgumentsHost, ExceptionFilter, INestApplication } from "@nestjs/common";
import { SwaggerModule, DocumentBuilder } from "@nestjs/swagger";
import { ConfigService } from "@nestjs/config";
import { AppModule } from "./app.module";

// Cloud Run configuration function
const configureCloudRunApp = async (
  app: INestApplication,
  serviceName: string,
  options?: {
    cors?: boolean;
    globalPrefix?: string;
  }
) => {
  const logger = new Logger(serviceName);

  // Configure port - Cloud Run injects PORT environment variable
  const port = process.env.PORT || 3001;

  // Configure CORS if enabled
  if (options?.cors) {
    app.enableCors();
  }

  // Set global prefix if provided
  if (options?.globalPrefix) {
    app.setGlobalPrefix(options.globalPrefix);
  }

  // Start listening
  await app.listen(port, "0.0.0.0");
  logger.log(`🚀 ${serviceName} is running on port ${port}`);

  // Graceful shutdown handling
  const signals = ["SIGTERM", "SIGINT"];
  signals.forEach((signal) => {
    process.on(signal, async () => {
      logger.log(`Received ${signal}, starting graceful shutdown...`);
      await app.close();
      logger.log(`${serviceName} terminated gracefully`);
      process.exit(0);
    });
  });

  // Handle unhandled promise rejections
  process.on("unhandledRejection", (reason, promise) => {
    logger.error("Unhandled Rejection at:", promise, "reason:", reason);
  });
};

interface ExtendedError extends Error {
  response?: { message: string };
  status?: number;
  code?: string;
  err?: any; // For RPC errors
}

@Catch()
class ErrorFilter implements ExceptionFilter {
  private readonly logger = new Logger("ErrorFilter");

  catch(error: ExtendedError, host: ArgumentsHost) {
    // Handle microservice timeouts
    if (error.code === 'ECONNREFUSED' || error.name === 'TimeoutError') {
      this.logger.error(`Microservice connection error: ${error.message}`);
      const ctx = host.switchToHttp();
      const response = ctx.getResponse();
      const request = ctx.getRequest();
      
      return response.status(503).json({
        statusCode: 503,
        timestamp: new Date().toISOString(),
        path: request.url,
        message: 'Service temporarily unavailable',
        error: 'Service Unavailable'
      });
    }

    // Handle RPC Exceptions from microservices
    if (error.err?.code === 'RPC_EXCEPTION') {
      const rpcError = error.err;
      this.logger.error(`RPC Exception:`, rpcError);
      const ctx = host.switchToHttp();
      const response = ctx.getResponse();
      const request = ctx.getRequest();
      
      return response.status(rpcError.status || 500).json({
        statusCode: rpcError.status || 500,
        timestamp: new Date().toISOString(),
        path: request.url,
        message: rpcError.message || 'Internal server error',
        error: rpcError.error || 'RPC Error'
      });
    }

    this.logger.error("Detailed error info:", {
      message: error.message,
      stack: error.stack,
      name: error.name,
      cause: (error as any).cause,
      metadata: (error as any).metadata,
    });

    const ctx = host.switchToHttp();
    const response = ctx.getResponse();
    const request = ctx.getRequest();

    // Log request details for debugging
    this.logger.debug("Request details:", {
      url: request.url,
      method: request.method,
      headers: request.headers,
      body: request.body,
      query: request.query,
    });

    // Handle validation errors (BadRequestException)
    if (error.name === "BadRequestException") {
      const status = 400;
      return response.status(status).json({
        statusCode: status,
        timestamp: new Date().toISOString(),
        path: request.url,
        message: error.message,
        // If validation error contains constraints, include them
        errors: error["response"]?.message || error.message,
      });
    }

    // Default error response for other types of errors
    let status = 500;
    
    if (typeof error["status"] === 'number') {
      status = error["status"];
    } else if (error.name === "NotFoundException") {
      status = 404;
    } else if (error.name === "UnauthorizedException") {
      status = 401;
    } else if (error.name === "ForbiddenException") {
      status = 403;
    }

    response.status(status).json({
      statusCode: status,
      timestamp: new Date().toISOString(),
      message: error.message || "Internal server error",
      path: request.url,
    });
  }
}

async function bootstrap() {
  const logger = new Logger("Gateway Service");

  const app = await NestFactory.create(AppModule, {
    logger: ["error", "warn", "log", "debug", "verbose"],
  });
  const configService = app.get(ConfigService);

  const apiPrefix = configService.get("API_PREFIX", "api/v1");
  const isDev = configService.get("NODE_ENV") === "development";

  // Add global error handler
  app.useGlobalFilters(new ErrorFilter());

  // Swagger documentation
  if (isDev) {
    const config = new DocumentBuilder()
      .setTitle("Gateway API")
      .setDescription("Gateway service API documentation")
      .setVersion("1.0")
      .addServer(`/${apiPrefix}`, 'Local environment') // Add server with API prefix
      .build();

    const document = SwaggerModule.createDocument(app, config);
    SwaggerModule.setup("docs", app, document);
  }

  // Configure for Cloud Run
  await configureCloudRunApp(app, "Gateway Service", {
    cors: true,
    globalPrefix: apiPrefix,
  });

  // Log configuration for debugging
  logger.debug("Gateway Service Configuration:", {
    apiPrefix,
    nodeEnv: isDev ? "development" : "production",
    corsEnabled: true,
    swaggerEnabled: isDev,
  });
}

// TODO: Add prometheus metrics
// TODO: Add request logging middleware
// TODO: Add error tracking integration

bootstrap().catch((error) => {
  const logger = new Logger("Gateway Service");
  logger.error("Failed to start gateway service:", error);
});
