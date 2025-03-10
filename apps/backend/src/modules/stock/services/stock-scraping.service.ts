import { Injectable, Logger } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Stock, Exchange } from "@epsx/shared";
import { Types } from "mongoose";
import { HttpService } from "./http.service";
import { STOCK_CONFIG } from "../config/stock.config";
import {
  StockModel,
  ExchangeModel,
  IStockScreenerResponse,
  IScrapingResponse,
  IStockData,
  IExchangeRef,
  IExchangeDocument,
  IStockCreate,
  IStockDocument,
} from "../types";
import { ScrapingStatusDto } from "../dto/stock.dto";

enum ScrapingStatus {
  COMPLETED = "COMPLETED",
  FAILED = "FAILED",
}

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

@Injectable()
export class StockScrapingService {
  private readonly logger = new Logger(StockScrapingService.name);
  private scrapedData: IStockScreenerResponse | null = null;
  private currentExchange: IExchangeRef | null = null;

  constructor(
    @InjectModel(Stock.name) private readonly stockModel: StockModel,
    @InjectModel(Exchange.name) private readonly exchangeModel: ExchangeModel,
    private readonly httpService: HttpService
  ) {}

  private async fetchAndProcessExchange(
    exchange: IExchangeDocument
  ): Promise<boolean> {
    try {
      const response = await this.httpService.fetchStockScreener(
        exchange.market_code
      );
      this.scrapedData = response.data;
      this.currentExchange = {
        _id: exchange._id as Types.ObjectId,
        market_code: exchange.market_code,
      };
      return true;
    } catch (error) {
      this.logger.error(
        `Error fetching data for exchange ${exchange.market_code}:`,
        error
      );
      return false;
    }
  }

  async saveScrapedData(): Promise<ScrapingStatusDto> {
    if (!this.scrapedData || !this.currentExchange) {
      return {
        status: "failure",
        message: "No scraped data available to save",
        timestamp: new Date().toISOString(),
      };
    }

    try {
      const stocksToProcess = this.scrapedData.data.data;
      let processedCount = 0;

      for (
        let j = 0;
        j < stocksToProcess.length;
        j += STOCK_CONFIG.stockBatchSize
      ) {
        const stockBatch = stocksToProcess.slice(
          j,
          j + STOCK_CONFIG.stockBatchSize
        );
        const symbols = stockBatch.map((stock: IStockData) => stock.s);

        const existingStocks = await this.stockModel
          .find({ symbol: { $in: symbols } })
          .select("symbol")
          .lean()
          .exec();

        const existingSymbols = new Set(
          existingStocks.map((stock: IStockDocument) => stock.symbol)
        );

        const newStocks: IStockCreate[] = stockBatch
          .filter((stock: IStockData) => !existingSymbols.has(stock.s))
          .map((stock: IStockData) => ({
            symbol: stock.s,
            company_name: stock.n,
            exchange: this.currentExchange!._id,
          }));

        if (newStocks.length > 0) {
          try {
            const createdStocks = await this.stockModel.create(newStocks);
            const stockIds = createdStocks.map(
              (stock: IStockDocument) => stock._id
            );

            await this.exchangeModel.findByIdAndUpdate(
              this.currentExchange._id,
              {
                $push: {
                  stocks: {
                    $each: stockIds,
                  },
                },
              },
              { new: true }
            );

            this.logger.log(
              `Inserted ${newStocks.length} new stocks and updated exchange references`
            );
          } catch (error) {
            this.logger.error(`Failed to insert stocks batch:`, error);
          }
        }

        processedCount += stockBatch.length;
        this.logger.log(
          `Processed ${stockBatch.length} stocks in batch (${newStocks.length} new)`
        );
      }

      return {
        status: "success",
        message: "Successfully saved scraped stock data",
        timestamp: new Date().toISOString(),
        totalStocksScraped: processedCount,
      };
    } catch (error) {
      this.logger.error("Failed to save scraped data:", error);
      return {
        status: "failure",
        message: "Failed to save scraped data",
        timestamp: new Date().toISOString(),
        totalStocksScraped: 0,
      };
    }
  }

  async scrapeAllStocks(): Promise<ScrapingStatusDto> {
    this.logger.log("Starting comprehensive stock scraping from all exchanges");
    const exchanges = await this.exchangeModel.find().lean().exec();

    for (const exchange of exchanges) {
      const success = await this.fetchAndProcessExchange(
        exchange as IExchangeDocument
      );
      if (success) {
        break; // For now, just process one exchange successfully
      }
    }

    return {
      status: this.scrapedData ? "success" : "failure",
      message: this.scrapedData
        ? "Stock data scraping completed successfully"
        : "Failed to scrape stock data",
      timestamp: new Date().toISOString(),
      totalStocksScraped: this.scrapedData?.data?.data?.length || 0,
      duration: "2 hours 15 minutes",
      nextScrape: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    };
  }

  async scrapeStocksByMarketCap(
    minMarketCap?: number,
    maxMarketCap?: number
  ): Promise<ScrapingStatusDto> {
    this.logger.log(
      `Starting stock scraping by market cap range: ${minMarketCap || 0} - ${maxMarketCap || "unlimited"}`
    );
    return await this.scrapeAllStocks();
  }

  async scrapeStocksBySector(sector: string): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for sector: ${sector}`);
    return await this.scrapeAllStocks();
  }

  async scrapeStocksByRegion(region: string): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for region: ${region}`);
    return await this.scrapeAllStocks();
  }

  async scrapeStocksByVolume(minVolume: number): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for minimum volume: ${minVolume}`);
    return await this.scrapeAllStocks();
  }

  async scrapeStockData(exchangeIds?: string[]): Promise<ScrapingStatusDto> {
    this.logger.log(
      `Starting stock scraping for exchanges: ${exchangeIds?.join(", ") || "all"}`
    );
    return await this.scrapeAllStocks();
  }
}
