import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { StockController } from "./stock.controller";
import { StockService } from "./stock.service";
import { HttpService } from "./services/http.service";
import {
  Stock,
  StockSchema,
  Exchange,
  ExchangeSchema,
} from "@epsx/shared";
import { ExchangeModule } from "../exchange/exchange.module";

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Stock.name, schema: StockSchema },
      {
        name: Exchange.name,
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
