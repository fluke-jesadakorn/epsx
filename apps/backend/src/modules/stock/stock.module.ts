import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { Stock, StockSchema } from '@epsx/shared';
import { Exchange, ExchangeSchema } from '@epsx/shared';
import { StockController } from './stock.controller';
import { StockService } from './stock.service';
import { StockDataService } from './services/stock-data.service';
import { StockScrapingService } from './services/stock-scraping.service';
import { HttpService } from './services/http.service';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Stock.name, schema: StockSchema },
      { name: Exchange.name, schema: ExchangeSchema }
    ])
  ],
  controllers: [StockController],
  providers: [
    StockService,
    StockDataService,
    StockScrapingService,
    HttpService
  ],
  exports: [
    StockService,
    StockDataService,
    StockScrapingService,
    MongooseModule
  ]
})
export class StockModule {}
