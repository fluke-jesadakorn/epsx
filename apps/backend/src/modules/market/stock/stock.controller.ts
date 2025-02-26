import { Controller, Get, Param } from '@nestjs/common';
import { ApiTags } from '@nestjs/swagger';
import { StockService } from './stock.service';

@ApiTags('Stock')
@Controller('stock')
export class StockController {
  constructor(private readonly stockService: StockService) {}

  @Get()
  async getAllStocks() {
    return this.stockService.getAllStocks();
  }

  @Get(':symbol')
  async getStockBySymbol(@Param('symbol') symbol: string) {
    return this.stockService.getStockBySymbol(symbol);
  }

  @Get(':symbol/price')
  async getStockPrice(@Param('symbol') symbol: string) {
    return this.stockService.getStockPrice(symbol);
  }
}
