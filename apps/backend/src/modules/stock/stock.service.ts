import { Injectable, Logger } from "@nestjs/common";
import {
  StockResponseDto,
  PaginatedStockResponse,
  StockPaginationParamsDto,
  ScrapingStatusDto,
} from "./dto/stock.dto";
import { StockDataService } from "./services/stock-data.service";
import { StockScrapingService } from "./services/stock-scraping.service";

@Injectable()
export class StockService {
  private readonly logger = new Logger(StockService.name);

  constructor(
    private readonly stockDataService: StockDataService,
    private readonly stockScrapingService: StockScrapingService
  ) {}

  async getAllStocks(
    params: StockPaginationParamsDto
  ): Promise<PaginatedStockResponse> {
    try {
      return await this.stockDataService.getAllStocks(params);
    } catch (error) {
      this.logger.error("Failed to get all stocks:", error);
      throw error;
    }
  }

  async getStocksByExchange(
    exchangeId: string,
    params: StockPaginationParamsDto
  ): Promise<PaginatedStockResponse> {
    try {
      return await this.stockDataService.getStocksByExchange(
        exchangeId,
        params
      );
    } catch (error) {
      this.logger.error(
        `Failed to get stocks for exchange ${exchangeId}:`,
        error
      );
      throw error;
    }
  }

  async getStockBySymbol(symbol: string): Promise<StockResponseDto> {
    try {
      return await this.stockDataService.getStockBySymbol(symbol);
    } catch (error) {
      this.logger.error(`Failed to get stock with symbol ${symbol}:`, error);
      throw error;
    }
  }

  async scrapeAllStocks(): Promise<ScrapingStatusDto> {
    try {
      const status = await this.stockScrapingService.scrapeAllStocks();
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error("Failed to scrape all stocks:", error);
      throw error;
    }
  }

  async scrapeStocksByMarketCap(
    minMarketCap: number,
    maxMarketCap: number
  ): Promise<ScrapingStatusDto> {
    try {
      const status = await this.stockScrapingService.scrapeStocksByMarketCap(
        minMarketCap,
        maxMarketCap
      );
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error("Failed to scrape stocks by market cap:", error);
      throw error;
    }
  }

  async scrapeStocksBySector(sector: string): Promise<ScrapingStatusDto> {
    try {
      const status =
        await this.stockScrapingService.scrapeStocksBySector(sector);
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error(`Failed to scrape stocks in sector ${sector}:`, error);
      throw error;
    }
  }

  async scrapeStocksByRegion(region: string): Promise<ScrapingStatusDto> {
    try {
      const status =
        await this.stockScrapingService.scrapeStocksByRegion(region);
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error(`Failed to scrape stocks in region ${region}:`, error);
      throw error;
    }
  }

  async scrapeStocksByVolume(minVolume: number): Promise<ScrapingStatusDto> {
    try {
      const status =
        await this.stockScrapingService.scrapeStocksByVolume(minVolume);
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error(
        `Failed to scrape stocks by volume ${minVolume}:`,
        error
      );
      throw error;
    }
  }

  async scrapeStockData(exchangeIds?: string[]): Promise<ScrapingStatusDto> {
    try {
      const status =
        await this.stockScrapingService.scrapeStockData(exchangeIds);
      if (status.status === "success") {
        await this.stockScrapingService.saveScrapedData();
      }
      return status;
    } catch (error) {
      this.logger.error("Failed to scrape stock data:", error);
      throw error;
    }
  }

  async saveStockData(): Promise<ScrapingStatusDto> {
    try {
      return await this.stockScrapingService.saveScrapedData();
    } catch (error) {
      this.logger.error("Failed to save stock data:", error);
      throw error;
    }
  }
}
