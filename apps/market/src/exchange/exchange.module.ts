import { Module } from "@nestjs/common";
import { ConfigModule, ConfigService } from "@nestjs/config";
import { MongooseModule } from "@nestjs/mongoose";
import { ExchangeController } from "./exchange.controller";
import { ExchangeService } from "./exchange.service";
import { ExchangeEntity, ExchangeSchema } from "@epsx/shared";

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
      { name: ExchangeEntity.name, schema: ExchangeSchema },
    ]),
  ],
  controllers: [ExchangeController],
  providers: [ExchangeService],
  exports: [
    ExchangeService,
    MongooseModule.forFeature([
      { name: ExchangeEntity.name, schema: ExchangeSchema },
    ]),
  ],
})
export class ExchangeModule {}
