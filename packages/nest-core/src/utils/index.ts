import { INestApplication } from '@nestjs/common';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';

export const setupSwagger = (
  app: INestApplication,
  title: string,
  description: string,
  version = '1.0',
) => {
  const config = new DocumentBuilder()
    .setTitle(title)
    .setDescription(description)
    .setVersion(version)
    .addBearerAuth()
    .build();

  const document = SwaggerModule.createDocument(app, config);
  SwaggerModule.setup('api', app, document);
};

export const validateEnvVars = (requiredVars: string[]) => {
  const missingVars = requiredVars.filter(
    varName => !process.env[varName],
  );

  if (missingVars.length > 0) {
    throw new Error(
      `Missing required environment variables: ${missingVars.join(', ')}`,
    );
  }
};

export const createMicroserviceOptions = (serviceName: string) => ({
  name: serviceName,
  transport: process.env.MESSAGE_BROKER_TRANSPORT || 'TCP',
  options: {
    host: process.env.MESSAGE_BROKER_HOST || 'localhost',
    port: parseInt(process.env.MESSAGE_BROKER_PORT || '3001', 10),
  },
});

// Add more utility functions as needed
