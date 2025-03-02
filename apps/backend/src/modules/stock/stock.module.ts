import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { StockController } from "./stock.controller";
import { StockService } from "./stock.service";
import { HttpService } from "./services/http.service";
import { Stock, StockSchema, Exchange, ExchangeSchema } from "@epsx/shared";
import { ExchangeModule } from "../exchange/exchange.module";

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Stock.name, schema: StockSchema },
      { name: Exchange.name, schema: ExchangeSchema },
    ]),
    ExchangeModule,
  ],
  controllers: [StockController],
  providers: [StockService, HttpService],
  exports: [StockService],
})
export class StockModule {}
