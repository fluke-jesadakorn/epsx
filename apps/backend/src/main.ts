// Testing file watching system
import { NestFactory } from "@nestjs/core";
import { Logger, ValidationPipe } from "@nestjs/common";
import { SwaggerModule, DocumentBuilder } from "@nestjs/swagger";
import { AppModule } from "./app.module";
import { ConfigService } from "@nestjs/config";
import cookieParser from 'cookie-parser';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Get config service
  const configService = app.get(ConfigService);
  const port = configService.get<number>("PORT", 3001);

  // Enable CORS with specific options
  // Enable CORS
  const frontendUrl = configService.get('FRONTEND_URL');
  const corsOrigins = [
    frontendUrl,
    'https://accounts.google.com',
    // Add localhost variations for development
    'http://localhost:3000',
    'http://localhost:4000'
  ];
  
  console.log('CORS Origins:', corsOrigins);
  
  app.enableCors({
    origin: corsOrigins,
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowedHeaders: ['Content-Type', 'Authorization', 'X-Requested-With'],
  });

  // Set global prefix with explicit path handling
  app.setGlobalPrefix("api/v1", {
    exclude: ["/health", "/docs", "/docs-json", "/docs/*"],
  });

  // Log environment configuration
  const backendUrl = configService.get('BACKEND_URL');
  const oauthCallbackUrl = new URL('/api/v1/auth/oauth/callback', backendUrl).toString();
  
  console.log('Environment Configuration:', {
    backendUrl,
    frontendUrl,
    oauthCallbackUrl
  });

  // Verify URLs are properly formatted
  try {
    new URL(backendUrl);
    new URL(frontendUrl);
    console.log('URLs are properly formatted');
  } catch (error) {
    console.error('Invalid URL format detected:', error);
  }

  // Enable cookie parser middleware
  app.use(cookieParser());

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
  SwaggerModule.setup("/docs", app, document, {
    useGlobalPrefix: false,
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
  Logger.log(`Application is running on: http://localhost:${port}`);
}

bootstrap();
