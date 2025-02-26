import { Module } from '@nestjs/common';
import { HttpModule } from '@nestjs/axios';
import { MarketController } from './market.controller';
import { MarketService } from './market.service';
import { ExchangeModule } from './exchange/exchange.module';
import { FinancialModule } from './financial/financial.module';
import { StockModule } from './stock/stock.module';

@Module({
  imports: [
    HttpModule,
    ExchangeModule,
    FinancialModule,
    StockModule,
  ],
  controllers: [MarketController],
  providers: [MarketService],
  exports: [MarketService],
})
export class MarketModule {}
