import { Module } from '@nestjs/common';
import { MongooseModule } from '@nestjs/mongoose';
import { FinancialController } from './financial.controller';
import { FinancialService } from './financial.service';
import { AggregationService } from './services/aggregation.service';
import { FinancialFetchService } from './services/financial-fetch.service';
import { ProcessingService } from './services/processing.service';
import { PaginationService } from './services/pagination.service';
import { EPSBatchProcessingService } from './services/eps-batch-processing.service';
import { 
  EpsGrowth, 
  EpsGrowthSchema,
  EPSGrowthProcessing,
  EPSGrowthProcessingSchema,
  EPSGrowthBatch,
  EPSGrowthBatchSchema,
  Stock,
  StockSchema 
} from '@epsx/shared';

import { StockModule } from '../stock/stock.module';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: EpsGrowth.name, schema: EpsGrowthSchema },
      { name: EPSGrowthProcessing.name, schema: EPSGrowthProcessingSchema },
      { name: EPSGrowthBatch.name, schema: EPSGrowthBatchSchema },
      { name: Stock.name, schema: StockSchema }
    ]),
    StockModule
  ],
  controllers: [FinancialController],
  providers: [
    FinancialService,
    AggregationService,
    FinancialFetchService,
    ProcessingService,
    PaginationService,
    EPSBatchProcessingService
  ],
  exports: [
    FinancialService,
    AggregationService,
    FinancialFetchService,
    ProcessingService,
    PaginationService,
    EPSBatchProcessingService
  ]
})
export class FinancialModule {}
