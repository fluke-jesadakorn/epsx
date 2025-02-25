import { Module } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import { StockController } from "./stock.controller";
import { StockService } from "./stock.service";
import { HttpService } from "./http.service";
import {
  Stock,
  StockSchema,
  Exchange,
  ExchangeSchema,
} from "@app/common/schemas";
import { ExchangeModule } from "@exchange/exchange.module";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: [".env"],
    }),
    MongooseModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: async (configService: ConfigService) => ({
        uri: configService.get<string>("MONGODB_URI"),
        dbName: configService.get<string>("MONGODB_DB_NAME"),
      }),
      inject: [ConfigService],
    }),
    MongooseModule.forFeature([
      { name: Stock.name, schema: StockSchema },
      {
        name: "Exchange",
        schema: ExchangeSchema,
      },
    ]),
    ExchangeModule,
  ],
  controllers: [StockController],
  providers: [StockService, HttpService],
})
export class StockModule {
  // TODO: Add lifecycle hooks for graceful shutdown
  // TODO: Add health check endpoints
  // TODO: Add telemetry and monitoring
  // TODO: Add caching layer
  // TODO: Add rate limiting
}

// Future enhancements:
// - Add WebSocket support for real-time updates
// - Implement circuit breaker for external API calls
// - Add retry mechanisms for database operations
// - Implement data validation pipeline
// - Add metrics collection for monitoring
