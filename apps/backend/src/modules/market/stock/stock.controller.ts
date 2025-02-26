import { Controller, Get } from "@nestjs/common";
import { Payload } from "@nestjs/microservices";
import { StockService } from "./stock.service";
import { PaginationParams } from "./interfaces/common.interfaces";

@Controller("stocks")
export class StockController {
  constructor(private readonly stockService: StockService) {}

  @Get()
  async getAllStocks(@Payload() params: PaginationParams) {
    return this.stockService.getAllStocks(params);
  }

  @Get("exchange/:exchangeId")
  async getStocksByExchange(
    @Payload() data: { exchangeId: string; params: PaginationParams }
  ) {
    return this.stockService.getStocksByExchange(data.exchangeId, data.params);
  }

  @Get("symbol/:symbol")
  async getStockBySymbol(@Payload() symbol: string) {
    return this.stockService.getStockBySymbol(symbol);
  }

  @Get("scrape")
  async scrapeAllStocks() {
    return this.stockService.scrapeAllStocks();
  }

  @Get("scrape/marketcap")
  async scrapeStocksByMarketCap(
    @Payload() data: { minMarketCap?: number; maxMarketCap?: number }
  ) {
    return this.stockService.scrapeStocksByMarketCap(
      data.minMarketCap,
      data.maxMarketCap
    );
  }

  @Get("scrape/sector/:sector")
  async scrapeStocksBySector(@Payload() sector: string) {
    return this.stockService.scrapeStocksBySector(sector);
  }

  @Get("scrape/region/:region")
  async scrapeStocksByRegion(@Payload() region: string) {
    return this.stockService.scrapeStocksByRegion(region);
  }

  @Get("scrape/volume/:minVolume")
  async scrapeStocksByVolume(@Payload() minVolume: number) {
    return this.stockService.scrapeStocksByVolume(minVolume);
  }

  @Get("scrape/exchanges")
  async scrapeStockData(@Payload() exchangeIds?: string[]) {
    return this.stockService.scrapeStockData(exchangeIds);
  }

  @Get("save")
  async saveStockData() {
    return this.stockService.saveStockData();
  }
}
