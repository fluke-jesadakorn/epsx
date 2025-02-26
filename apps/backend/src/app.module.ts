import { Module } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import { MarketModule } from "./modules/market/market.module";
import { TerminusModule } from "@nestjs/terminus";
import { HttpModule } from "@nestjs/axios";
import { FinancialModule } from "./modules/market/financial/financial.module";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),

    MongooseModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: async (configService: ConfigService) => ({
        uri: configService.get<string>("MONGODB_URI"),
      }),
      inject: [ConfigService],
    }),

    TerminusModule,
    HttpModule,

    FinancialModule,
    MarketModule,
  ],
})
export class AppModule {}
