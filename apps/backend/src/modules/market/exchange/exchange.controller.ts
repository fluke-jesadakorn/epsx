import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  Query,
} from "@nestjs/common";
import { ExchangeService } from "./exchange.service";
import { ExchangeEntity } from "@epsx/shared";

@Controller("exchanges")
export class ExchangeController {
  constructor(private readonly exchangeService: ExchangeService) {}

  @Post()
  async create(@Body() exchangeData: Partial<ExchangeEntity>) {
    return this.exchangeService.create(exchangeData);
  }

  @Get()
  async findAll(@Query("page") page: number, @Query("limit") limit: number) {
    const skip = (page - 1) * limit;
    return this.exchangeService.findAll(skip, +limit);
  }

  @Get(":marketCode")
  async findOne(@Param("marketCode") marketCode: string) {
    return this.exchangeService.findOne(marketCode);
  }

  @Put(":marketCode")
  async update(
    @Param("marketCode") marketCode: string,
    @Body() updateData: Partial<ExchangeEntity>
  ) {
    return this.exchangeService.update(marketCode, updateData);
  }

  @Delete(":marketCode")
  async remove(@Param("marketCode") marketCode: string) {
    return this.exchangeService.remove(marketCode);
  }

  @Post("scrape")
  async scrapeExchanges() {
    return this.exchangeService.scrapeAndSaveExchanges();
  }
}
