import { NestFactory } from "@nestjs/core";
import { Logger, INestApplication, Module } from "@nestjs/common";
import { ConfigModule } from "@nestjs/config";
import { HealthController } from "./health.controller";

const logger = new Logger("AI");

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
  ],
  controllers: [HealthController],
  providers: [],
})
class AppModule {}

export async function createApp(): Promise<INestApplication> {
  const app = await NestFactory.create(AppModule, {
    logger: ["error", "warn", "log", "debug", "verbose"],
    bodyParser: true,
  });

  // Enable CORS for Cloud Run
  app.enableCors();

  return app;
}
