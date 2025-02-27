import { Module } from '@nestjs/common';
import { HttpModule } from '@nestjs/axios';
import { MongooseModule } from '@nestjs/mongoose';
import { MarketController } from './market.controller';
import { MarketService } from './market.service';
import { ExchangeModule } from './exchange/exchange.module';
import { FinancialModule } from './financial/financial.module';
import { StockModule } from './stock/stock.module';
import { EpsGrowth, EpsGrowthSchema } from '@epsx/shared';

@Module({
  imports: [
    HttpModule,
    MongooseModule.forFeature([
      { name: EpsGrowth.name, schema: EpsGrowthSchema }
    ]),
    ExchangeModule,
    FinancialModule,
    StockModule,
  ],
  controllers: [MarketController],
  providers: [MarketService],
  exports: [MarketService],
})
export class MarketModule {}
