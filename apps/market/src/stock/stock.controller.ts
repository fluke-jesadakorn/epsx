import {
  Controller,
  Get,
  Param,
  Query,
  Post,
  Body,
} from "@nestjs/common";
import { StockService } from "./stock.service";
import { PaginationParams } from "./interfaces/common.interfaces";

@Controller('stocks')
export class StockController {
  constructor(private readonly stockService: StockService) {}

  @Get()
  async getAllStocks(@Query() params: PaginationParams) {
    return this.stockService.getAllStocks(params);
  }

  @Get('exchange/:exchangeId')
  async getStocksByExchange(
    @Param('exchangeId') exchangeId: string,
    @Query() params: PaginationParams
  ) {
    return this.stockService.getStocksByExchange(exchangeId, params);
  }

  @Get('symbol/:symbol')
  async getStockBySymbol(@Param('symbol') symbol: string) {
    return this.stockService.getStockBySymbol(symbol);
  }

  @Post('save')
  async saveStockData() {
    return this.stockService.saveStockData();
  }

  @Post('scrape')
  async scrapeStockData(@Body('exchangeIds') exchangeIds?: string[]) {
    return this.stockService.scrapeStockData(exchangeIds);
  }

  @Post('scrape/all')
  async scrapeAllStocks() {
    // TODO: Add check for exchanges setup before scraping
    // Consider checking exchange collection and populating if empty
    return this.stockService.scrapeAllStocks();
  }

  @Post('scrape/market-cap')
  async scrapeStocksByMarketCap(
    @Body() data: { minMarketCap?: number; maxMarketCap?: number }
  ) {
    return this.stockService.scrapeStocksByMarketCap(data.minMarketCap, data.maxMarketCap);
  }

  @Post('scrape/sector/:sector')
  async scrapeStocksBySector(@Param('sector') sector: string) {
    return this.stockService.scrapeStocksBySector(sector);
  }

  @Post('scrape/region/:region')
  async scrapeStocksByRegion(@Param('region') region: string) {
    return this.stockService.scrapeStocksByRegion(region);
  }

  @Post('scrape/volume')
  async scrapeStocksByVolume(@Body('minVolume') minVolume: number) {
    return this.stockService.scrapeStocksByVolume(minVolume);
  }

  // TODO: Add endpoints for:
  // - Real-time price updates
  // - Historical data queries
  // - Technical analysis requests
  // - Market indicators
  // - Stock performance metrics
}
