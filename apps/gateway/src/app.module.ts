import { Module } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { ClientsModule, Transport } from "@nestjs/microservices";
import { TerminusModule } from "@nestjs/terminus";
import { FirebaseConfigModule } from "./config/firebase.config";
import { FirebaseService } from "./services/firebase.service";
import { MarketController } from "./controllers/market.controller";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: [".env"],
    }),
    FirebaseConfigModule,
    ClientsModule.registerAsync([
      {
        name: "MARKET_SERVICE",
        imports: [ConfigModule],
        useFactory: (configService: ConfigService) => ({
          transport: Transport.TCP,
          options: {
            host: configService.get("MARKET_SERVICE_HOST", "localhost"),
            port: configService.get("MARKET_SERVICE_PORT", 3002),
            retryAttempts: 5,
            retryDelay: 3000,
            timeout: 10000,
            reconnectDelay: 5000,
            keepalive: true,
            keepaliveTimeout: 5000,
          },
        }),
        inject: [ConfigService],
      },
      {
        name: "AI_SERVICE",
        imports: [ConfigModule],
        useFactory: (configService: ConfigService) => ({
          transport: Transport.TCP,
          options: {
            host: configService.get("AI_SERVICE_HOST", "localhost"),
            port: configService.get("AI_SERVICE_PORT", 3003),
            retryAttempts: 5,
            retryDelay: 3000,
            timeout: 10000,
            reconnectDelay: 5000,
            keepalive: true,
            keepaliveTimeout: 5000,
          },
        }),
        inject: [ConfigService],
      },
    ]),
    TerminusModule,
  ],
  controllers: [MarketController],
  providers: [FirebaseService],
})
export class AppModule {
  // TODO: Add global exception filters
  // TODO: Add request logging
  // TODO: Add metrics collection
  // TODO: Add authentication guard
}

// Future enhancements:
// - Implement circuit breaker for microservice calls
// - Add request/response transformation interceptors
// - Add caching layer
// - Implement rate limiting
// - Add API versioning support
// - Add request validation pipe
