import { NestFactory } from '@nestjs/core';
import { ValidationPipe } from '@nestjs/common';
import { SwaggerModule, DocumentBuilder } from '@nestjs/swagger';
import { AppModule } from './app.module';
import { ConfigService } from '@nestjs/config';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Get config service
  const configService = app.get(ConfigService);
  const port = configService.get<number>('PORT', 3001);

  // Enable CORS
  app.enableCors();

  // Enable validation pipes
  app.useGlobalPipes(new ValidationPipe({ transform: true }));

  // Swagger documentation setup
  const config = new DocumentBuilder()
    .setTitle('EPSX Backend API')
    .setDescription('Combined backend service API documentation')
    .setVersion('1.0')
    .addBearerAuth()
    .build();

  const document = SwaggerModule.createDocument(app, config);
  SwaggerModule.setup('api', app, document);

  await app.listen(port);
  console.log(`Application is running on: http://localhost:${port}`);
}

bootstrap();
