import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { ConfigModule } from "@nestjs/config";
import { HttpModule } from "@nestjs/axios";
import { MarketController } from "./market.controller";
import { MarketService } from "./market.service";
import { EpsGrowth, EpsGrowthSchema } from "@epsx/shared";
import { APP_GUARD } from "@nestjs/core";
import { RolesGuard } from "../../shared/guards/role.guard";
import { FinancialModule } from "../financial/financial.module";
import { ExchangeModule } from "../exchange/exchange.module";
import { StockModule } from "../stock/stock.module";

@Module({
  imports: [
    ConfigModule,
    HttpModule,
    MongooseModule.forFeature([
      { name: EpsGrowth.name, schema: EpsGrowthSchema },
    ]),
    FinancialModule,
    StockModule,
    ExchangeModule,
  ],
  controllers: [MarketController],
  providers: [
    MarketService,
    {
      provide: APP_GUARD,
      useClass: RolesGuard,
    },
  ],
  exports: [MarketService],
})
export class MarketModule {}
