import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { Logger } from '@nestjs/common';
import { configureCloudRunApp } from '@epsx/shared';

async function bootstrap() {
  const logger = new Logger('Scheduler Service');
  const app = await NestFactory.create(AppModule, {
    logger: ['error', 'warn', 'log', 'debug', 'verbose'],
  });

  await configureCloudRunApp(app, 'Scheduler Service', {
    cors: true,
    globalPrefix: 'api/v1'
  });
}

bootstrap().catch((error) => {
  const logger = new Logger('Scheduler Service');
  logger.error('Failed to start scheduler service:', error);
  process.exit(1);
});
