import { Controller, Get, Post, Put, Delete, Body, Param, Query, HttpStatus, HttpCode } from "@nestjs/common";
import { ExchangeService } from "./exchange.service";
import { IExchange } from "@epsx/shared";

@Controller('exchanges')
export class ExchangeController {
  constructor(private readonly exchangeService: ExchangeService) {}

  @Post()
  @HttpCode(HttpStatus.CREATED)
  async create(@Body() exchangeData: Partial<IExchange>) {
    return this.exchangeService.create(exchangeData);
  }

  @Get()
  async findAll(@Query('page') page = 1, @Query('limit') limit = 10) {
    const skip = (page - 1) * limit;
    return this.exchangeService.findAll(skip, +limit);
  }

  @Get(':marketCode')
  async findOne(@Param('marketCode') marketCode: string) {
    return this.exchangeService.findOne(marketCode);
  }

  @Put(':marketCode')
  async update(
    @Param('marketCode') marketCode: string,
    @Body() updateData: Partial<IExchange>
  ) {
    return this.exchangeService.update(marketCode, updateData);
  }

  @Delete(':marketCode')
  @HttpCode(HttpStatus.NO_CONTENT)
  async remove(@Param('marketCode') marketCode: string) {
    return this.exchangeService.remove(marketCode);
  }

  @Post('scrape')
  @HttpCode(HttpStatus.OK)
  async scrapeExchanges() {
    return this.exchangeService.scrapeAndSaveExchanges();
  }
}
