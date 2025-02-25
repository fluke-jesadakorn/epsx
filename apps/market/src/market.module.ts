import { Module } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import { StockModule } from "./stock/stock.module";
import { ExchangeModule } from "./exchange/exchange.module";

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
      envFilePath: [".env"],
    }),
    MongooseModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: async (configService: ConfigService) => ({
        uri:
          configService.get<string>("MONGODB_URI") ||
          "mongodb://localhost:27017",
        dbName: configService.get<string>("MONGODB_DB_NAME") || "epsx",
      }),
      inject: [ConfigService],
    }),
    StockModule,
    ExchangeModule,
  ],
})
export class MarketModule {}
