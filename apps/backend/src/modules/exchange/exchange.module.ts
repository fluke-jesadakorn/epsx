import { Module } from "@nestjs/common";
import { MongooseModule } from "@nestjs/mongoose";
import { ExchangeController } from "./exchange.controller";
import { ExchangeService } from "./exchange.service";
import { Exchange, ExchangeSchema } from "@epsx/shared";

// TODO: Future Feature - Add caching layer to reduce database load
// TODO: Future Feature - Add rate limiting for scraping to avoid being blocked
@Module({
  imports: [
    MongooseModule.forFeature([
      { name: Exchange.name, schema: ExchangeSchema },
    ]),
  ],
  controllers: [ExchangeController],
  providers: [ExchangeService],
  exports: [
    ExchangeService,
    MongooseModule.forFeature([
      { name: Exchange.name, schema: ExchangeSchema },
    ]),
  ],
})
export class ExchangeModule {}
