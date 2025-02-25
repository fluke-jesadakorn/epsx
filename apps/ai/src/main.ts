import { Logger, INestApplication } from "@nestjs/common";
import { createApp } from "./bootstrap";

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
  const port = process.env.PORT || 4000;

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
  const logger = new Logger("AI Service");
  const app = await createApp();

  await configureCloudRunApp(app, "AI Service", {
    cors: true,
    globalPrefix: "api/v1"
  });
}

bootstrap().catch((error) => {
  const logger = new Logger("AI Service");
  logger.error("Failed to start AI service:", error);
  process.exit(1);
});
