import { NestFactory } from "@nestjs/core";
import { ValidationPipe, Logger, INestApplication } from "@nestjs/common";
import { Transport } from "@nestjs/microservices";
import { MarketModule } from "./market.module";

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
  const port = process.env.PORT || 3002;

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

async function bootstrap() {
  const logger = new Logger("Market Service");

  const app = await NestFactory.create(MarketModule);

  const microservice = app.connectMicroservice({
    transport: Transport.TCP,
    options: {
      host: process.env.MARKET_SERVICE_HOST || "localhost",
      port: parseInt(process.env.MARKET_SERVICE_TCP_PORT || "3002", 10),
    },
  });

  app.useGlobalPipes(new ValidationPipe());

  await microservice.listen();
  logger.log("TCP Microservice is listening");

  await configureCloudRunApp(app, "Market Service", {
    cors: true,
    globalPrefix: "api/v1",
  });
}

bootstrap().catch((error) => {
  const logger = new Logger("Market Service");
  logger.error("Failed to start market service:", error);
  process.exit(1);
});
