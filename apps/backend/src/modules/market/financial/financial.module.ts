import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { FinancialController } from "./financial.controller";
import { FinancialService } from "./financial.service";
import {
  Financial,
  FinancialSchema,
  Stock,
  StockSchema,
  EpsGrowth,
  EpsGrowthSchema,
  EPSGrowthProcessing,
  EPSGrowthProcessingSchema,
  EPSGrowthBatch,
  EPSGrowthBatchSchema,
} from "@epsx/shared";
import { FinancialFetchService } from "./services/financial-fetch.service";
import { WorkerPoolService } from "./services/worker-pool.service";
import { FetchStateService } from "./services/fetch-state.service";
import { HttpService } from "./services/http.service";
import { EPSBatchProcessingService } from "./services/eps-batch-processing.service";
import { AggregationService } from "./services/aggregation.service";

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Financial.name, schema: FinancialSchema },
      { name: Stock.name, schema: StockSchema },
      { name: EpsGrowth.name, schema: EpsGrowthSchema },
      { name: EPSGrowthProcessing.name, schema: EPSGrowthProcessingSchema },
      { name: EPSGrowthBatch.name, schema: EPSGrowthBatchSchema },
    ]),
  ],
  controllers: [FinancialController],
  providers: [
    FinancialService,
    FinancialFetchService,
    WorkerPoolService,
    FetchStateService,
    HttpService,
    EPSBatchProcessingService,
    AggregationService,
  ],
  exports: [
    FinancialService,
    FinancialFetchService,
    WorkerPoolService,
    FetchStateService,
    HttpService,
    EPSBatchProcessingService,
    AggregationService,
  ],
})
export class FinancialModule {}
