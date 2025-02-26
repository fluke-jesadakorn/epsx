import { NestFactory } from "@nestjs/core";
import { ValidationPipe } from "@nestjs/common";
import { SwaggerModule, DocumentBuilder } from "@nestjs/swagger";
import { AppModule } from "./app.module";
import { ConfigService } from "@nestjs/config";

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Get config service
  const configService = app.get(ConfigService);
  const port = configService.get<number>("PORT", 3001);

  // Enable CORS
  app.enableCors();

  // Enable validation pipes
  app.useGlobalPipes(new ValidationPipe({ transform: true }));

  // Swagger documentation setup
  const config = new DocumentBuilder()
    .setTitle("EPSX Backend API")
    .setDescription(
      `
      Comprehensive API documentation for the EPSX Backend Service.
      This service provides endpoints for market data, AI analysis, and user management.
    `
    )
    .setVersion("1.0.0")
    .setContact("EPSX Team", "https://epsx.com", "support@epsx.com")
    .setTermsOfService("https://epsx.com/terms")
    .setExternalDoc("Additional Documentation", "https://docs.epsx.com")
    .addTag("Health", "Service health and monitoring endpoints")
    .addBearerAuth(
      {
        type: "http",
        scheme: "bearer",
        bearerFormat: "JWT",
        description: "Enter your JWT token",
      },
      "JWT-auth"
    )
    .addApiKey(
      {
        type: "apiKey",
        name: "x-api-key",
        in: "header",
        description: "API key for service-to-service communication",
      },
      "api-key"
    )
    .build();

  const document = SwaggerModule.createDocument(app, config);

  // Customize Swagger UI
  SwaggerModule.setup("docs", app, document, {
    swaggerOptions: {
      persistAuthorization: true,
      tagsSorter: "alpha",
      operationsSorter: "alpha",
      docExpansion: "none",
      filter: true,
      showExtensions: true,
    },
    customCss: ".swagger-ui .topbar { display: none }",
    customSiteTitle: "EPSX API Documentation",
  });

  await app.listen(port);
  console.log(`Application is running on: http://localhost:${port}`);
}

bootstrap();
