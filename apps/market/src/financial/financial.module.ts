import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { FinancialService } from "./financial.service";
import { FinancialController } from "./financial.controller";
import { AggregationService } from "./aggregation.service";
import { CacheService } from "./services/cache.service";
import { EPSBatchProcessingService } from "./services/eps-batch-processing.service";

import {
  Financial,
  FinancialSchema,
  Stock,
  StockSchema,
  EPSGrowthProcessing,
  EPSGrowthProcessingSchema,
  EPSGrowthBatch,
  EPSGrowthBatchSchema,
  EpsGrowth,
  EpsGrowthSchema,
} from "@app/common/schemas";

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
      { name: Financial.name, schema: FinancialSchema },
      { name: Stock.name, schema: StockSchema },
      { name: EPSGrowthProcessing.name, schema: EPSGrowthProcessingSchema },
      { name: EPSGrowthBatch.name, schema: EPSGrowthBatchSchema },
      { name: EpsGrowth.name, schema: EpsGrowthSchema },
    ]),
  ],
  controllers: [FinancialController],
  providers: [
    FinancialService,
    AggregationService,
    CacheService,
    EPSBatchProcessingService,
  ],
})
export class FinancialModule {}
