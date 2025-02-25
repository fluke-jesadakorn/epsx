import {
  BadRequestException,
  Injectable,
  Logger,
  NotFoundException,
} from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Model } from "mongoose";
import { Stock, Exchange } from "@app/common/schemas";
import { HttpService } from "./http.service";
import {
  StockScreenerResponse,
  PaginationParams,
  Paginate,
  formatPaginationResponse,
} from "./interfaces/common.interfaces";

const STOCK_CONFIG = {
  stockBatchSize: 100,
  maxParallelRequests: 3,
  batchDelay: 1000,
};

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

@Injectable()
export class StockService {
  private readonly logger = new Logger(StockService.name);

  constructor(
    @InjectModel(Stock.name)
    private stockModel: Model<Stock>,
    @InjectModel("Exchange")
    private exchangeModel: Model<Exchange>,
    private readonly httpService: HttpService
  ) {}

  /**
   * Get all stocks with pagination
   */
  @Paginate()
  async getAllStocks(params: PaginationParams = {}) {
    const skip = params.skip || 0;
    const limit = params.limit || 20;
    const [data, total] = await Promise.all([
      this.stockModel
        .find()
        .populate("exchange")
        .skip(skip)
        .limit(limit)
        .exec(),
      this.stockModel.countDocuments().exec(),
    ]);
    return formatPaginationResponse(data, total, skip, limit);
  }

  /**
   * Get stocks by exchange with pagination
   */
  @Paginate()
  async getStocksByExchange(exchangeId: string, params: PaginationParams = {}) {
    if (!exchangeId) {
      throw new BadRequestException("Exchange ID is required");
    }

    const skip = params.skip || 0;
    const limit = params.limit || 20;
    const [data, total] = await Promise.all([
      this.stockModel
        .find({ exchange: exchangeId })
        .populate("exchange")
        .skip(skip)
        .limit(limit)
        .exec(),
      this.stockModel.countDocuments({ exchange: exchangeId }).exec(),
    ]);

    if (!data.length) {
      throw new NotFoundException(`No stocks found for exchange ${exchangeId}`);
    }

    return formatPaginationResponse(data, total, skip, limit);
  }

  /**
   * Get stock by symbol
   */
  async getStockBySymbol(symbol: string) {
    const stock = await this.stockModel
      .findOne({ symbol })
      .populate("exchange")
      .exec();

    if (!stock) {
      throw new NotFoundException(`Stock with symbol ${symbol} not found`);
    }

    return stock;
  }

  /**
   * Scrape stock data from all exchanges and save to database
   *
   * Future feature possibilities:
   * - Add support for real-time stock price updates via WebSocket
   * - Implement historical data storage and tracking
   * - Add data validation and sanitization for more robust error handling
   * - Include additional stock metrics (market cap, volume, P/E ratio, etc.)
   * - Add retry mechanism for failed exchange requests
   * - Implement selective exchange update (update specific exchanges only)
   * - Add data normalization for cross-exchange compatibility
   * - Consider implementing a caching layer for frequently accessed data
   */
  async saveStockData() {
    this.logger.log("Starting stock data processing for all exchanges");

    const existingExchanges = await this.exchangeModel.find().exec();

    try {
      for (
        let i = 0;
        i < existingExchanges.length;
        i += STOCK_CONFIG.maxParallelRequests
      ) {
        const batch = existingExchanges.slice(
          i,
          i + STOCK_CONFIG.maxParallelRequests
        );
        this.logger.log(
          `Processing batch ${Math.floor(i / STOCK_CONFIG.maxParallelRequests) + 1} of ${Math.ceil(existingExchanges.length / STOCK_CONFIG.maxParallelRequests)}`
        );

        // Process each exchange in the batch concurrently
        const batchPromises = batch.map(async (exchange) => {
          if (!exchange._id) {
            this.logger.error(`Exchange ${exchange.market_code} has no ID`);
            return;
          }

          try {
            const stockData =
              await this.httpService.fetchStockScreener<StockScreenerResponse>(
                exchange.market_code
              );

            if (!stockData) {
              this.logger.error(
                `Failed to fetch data for exchange ${exchange.market_code}`
              );
              return;
            }

            // Validate data structure and extract stocks array
            if (!stockData.data?.data) {
              this.logger.error(
                `Invalid data structure received for exchange ${exchange.market_code}`
              );
              return;
            }

            const stocksToProcess = stockData.data.data;
            const totalStocks =
              stockData.data.resultsCount || stocksToProcess.length;
            this.logger.log(
              `Retrieved ${totalStocks} stocks for ${exchange.market_code}`
            );

            let processedCount = 0;

            // Process stocks in batches
            for (
              let j = 0;
              j < stocksToProcess.length;
              j += STOCK_CONFIG.stockBatchSize
            ) {
              const stockBatch = stocksToProcess.slice(
                j,
                j + STOCK_CONFIG.stockBatchSize
              );

              // Create array of symbols for bulk existence check
              const symbols = stockBatch.map((s) => s.s);

              // Bulk check for existing stocks
              const existingStocks = await this.stockModel
                .find({ symbol: { $in: symbols } })
                .select("symbol")
                .exec();

              const existingSymbols = new Set(
                existingStocks.map((s) => s.symbol)
              );

              // Filter only new stocks that need to be inserted
              const newStocks = stockBatch
                .filter((s) => !existingSymbols.has(s.s))
                .map((s) => ({
                  symbol: s.s,
                  company_name: s.n,
                  exchange: exchange._id, // Add exchange relationship
                }));

              // Bulk insert new stocks
              if (newStocks.length > 0) {
                try {
                  // Update exchange document to include new stock references
                  const createdStocks = await this.stockModel.create(newStocks);

                  // Update exchange document with the new stock references
                  await this.exchangeModel.findByIdAndUpdate(
                    exchange._id,
                    {
                      $push: {
                        stocks: {
                          $each: createdStocks.map((stock) => stock._id),
                        },
                      },
                    },
                    { new: true }
                  );

                  this.logger.log(
                    `Inserted ${newStocks.length} new stocks and updated exchange references`
                  );
                } catch (error) {
                  this.logger.error(
                    `Failed to insert stocks batch: ${error.message}`
                  );
                  // TODO: Add rollback mechanism for partial failures
                }
              }

              processedCount += stockBatch.length;
              this.logger.log(
                `Processed ${stockBatch.length} stocks in batch (${newStocks.length} new)`
              );
            }

            this.logger.log(
              `Exchange ${exchange.market_code} processing completed. Processed ${processedCount} stocks.`
            );
          } catch (error) {
            this.logger.error(
              `Error processing exchange ${exchange.market_code}: ${error.message}`
            );
            // TODO: Implement retry mechanism for failed exchange requests
          }
        });

        await Promise.all(batchPromises);

        if (i + STOCK_CONFIG.maxParallelRequests < existingExchanges.length) {
          this.logger.log(
            `Waiting ${STOCK_CONFIG.batchDelay}ms before processing next batch...`
          );
          await sleep(STOCK_CONFIG.batchDelay);
        }
      }

      this.logger.log("All batches have been processed successfully.");
    } catch (error) {
      this.logger.error("Fatal error during stock data processing");
    }
  }

  /**
   * Scrape stock data from exchanges with improved error handling and performance
   * Similar to saveStockData but with additional features and optimizations
   *
   * @param exchangeIds Optional array of exchange IDs to scrape. If not provided, scrapes all exchanges.
   * @returns Summary of the scraping operation
   *
   * TODO: Future Improvements
   * 1. Add atomic transactions for consistent data
   * 2. Implement bulk operations for better performance
   * 3. Add validation for duplicate stock references
   * 4. Add cascade delete functionality
   * 5. Implement periodic reference integrity checks
   * 6. Add batch size configuration for reference updates
   * 7. Implement rollback mechanism for failed operations
   * 8. Add caching for frequently accessed relationships
   * 9. Implement query optimization for relationship lookups
   * 10. Add monitoring for relationship health metrics
   */
  async scrapeStockData(exchangeIds?: string[]) {
    this.logger.log("Starting stock data scraping process");

    let existingExchanges;
    try {
      if (exchangeIds?.length) {
        existingExchanges = await this.exchangeModel
          .find({
            _id: { $in: exchangeIds },
          })
          .exec();
        if (!existingExchanges.length) {
          throw new NotFoundException("No exchanges found with provided IDs");
        }
      } else {
        existingExchanges = await this.exchangeModel.find().exec();
        if (!existingExchanges.length) {
          throw new NotFoundException(
            "No exchanges found in database. Please ensure exchanges are populated before scraping stocks."
          );
        }
      }
    } catch (error) {
      this.logger.error(`Failed to fetch exchanges: ${error.message}`);
      throw new BadRequestException("Failed to fetch exchanges");
    }

    const summary = {
      totalExchanges: existingExchanges.length,
      processedExchanges: 0,
      totalStocks: 0,
      newStocks: 0,
      failedExchanges: 0,
      errors: [] as string[],
    };

    try {
      for (
        let i = 0;
        i < existingExchanges.length;
        i += STOCK_CONFIG.maxParallelRequests
      ) {
        const batch = existingExchanges.slice(
          i,
          i + STOCK_CONFIG.maxParallelRequests
        );
        this.logger.log(
          `Processing batch ${Math.floor(i / STOCK_CONFIG.maxParallelRequests) + 1} of ${Math.ceil(existingExchanges.length / STOCK_CONFIG.maxParallelRequests)}`
        );

        const batchResults = await Promise.allSettled(
          batch.map(async (exchange) => {
            if (!exchange._id) {
              throw new Error(`Exchange ${exchange.market_code} has no ID`);
            }

            const stockData =
              await this.httpService.fetchStockScreener<StockScreenerResponse>(
                exchange.market_code
              );

            if (!stockData?.data?.data) {
              throw new Error(
                `Invalid data structure received for exchange ${exchange.market_code}`
              );
            }

            const stocksToProcess = stockData.data.data;
            const exchangeStats = {
              processedStocks: 0,
              newStocks: 0,
            };

            for (
              let j = 0;
              j < stocksToProcess.length;
              j += STOCK_CONFIG.stockBatchSize
            ) {
              const stockBatch = stocksToProcess.slice(
                j,
                j + STOCK_CONFIG.stockBatchSize
              );
              const symbols = stockBatch.map((s) => s.s);

              const existingStocks = await this.stockModel
                .find({ symbol: { $in: symbols } })
                .select("symbol")
                .exec();

              const existingSymbols = new Set(
                existingStocks.map((s) => s.symbol)
              );
              const newStocks = stockBatch
                .filter((s) => !existingSymbols.has(s.s))
                .map((s) => ({
                  symbol: s.s,
                  company_name: s.n,
                  exchange: exchange._id,
                }));

              if (newStocks.length > 0) {
                const createdStocks = await this.stockModel.create(newStocks);
                await this.exchangeModel.findByIdAndUpdate(
                  exchange._id,
                  {
                    $push: {
                      stocks: {
                        $each: createdStocks.map((stock) => stock._id),
                      },
                    },
                  },
                  { new: true }
                );
                exchangeStats.newStocks += newStocks.length;
              }
              exchangeStats.processedStocks += stockBatch.length;
            }

            return {
              exchange: exchange.market_code,
              ...exchangeStats,
            };
          })
        );

        // Process batch results
        batchResults.forEach((result) => {
          if (result.status === "fulfilled") {
            summary.processedExchanges++;
            summary.totalStocks += result.value.processedStocks;
            summary.newStocks += result.value.newStocks;
          } else {
            summary.failedExchanges++;
            summary.errors.push(result.reason.message);
          }
        });

        if (i + STOCK_CONFIG.maxParallelRequests < existingExchanges.length) {
          this.logger.log(
            `Waiting ${STOCK_CONFIG.batchDelay}ms before next batch...`
          );
          await sleep(STOCK_CONFIG.batchDelay);
        }
      }

      this.logger.log("Stock scraping completed", summary);
      return summary;
    } catch (error) {
      this.logger.error("Fatal error during stock scraping", error);
      throw new Error("Stock scraping failed");
    }
  }

  /**
   * Scrape stocks by market cap range
   * @param minMarketCap Minimum market cap in millions
   * @param maxMarketCap Maximum market cap in millions
   */
  async scrapeStocksByMarketCap(minMarketCap?: number, maxMarketCap?: number) {
    this.logger.log(
      `Starting stock scraping by market cap range: ${minMarketCap || 0} - ${maxMarketCap || "unlimited"}`
    );
    return this.scrapeStockData(); // TODO: Add market cap filtering
  }

  /**
   * Scrape stocks by sector
   * @param sector Sector to filter by
   */
  async scrapeStocksBySector(sector: string) {
    this.logger.log(`Starting stock scraping for sector: ${sector}`);
    return this.scrapeStockData(); // TODO: Add sector filtering
  }

  /**
   * Scrape stocks by region
   * @param region Geographic region to filter by
   */
  async scrapeStocksByRegion(region: string) {
    this.logger.log(`Starting stock scraping for region: ${region}`);
    return this.scrapeStockData(); // TODO: Add region filtering
  }

  /**
   * Scrape all stocks from all exchanges
   * This is a comprehensive scrape that ensures complete coverage
   */
  async scrapeAllStocks() {
    this.logger.log("Starting comprehensive stock scraping from all exchanges");
    return this.scrapeStockData();
  }

  /**
   * Scrape stocks by volume
   * @param minVolume Minimum trading volume
   */
  async scrapeStocksByVolume(minVolume: number) {
    this.logger.log(`Starting stock scraping for minimum volume: ${minVolume}`);
    return this.scrapeStockData(); // TODO: Add volume filtering
  }

  // TODO: Add methods for:
  // - Real-time price updates
  // - Historical data retrieval
  // - Technical analysis
  // - Market indicators
  // - Stock performance metrics
}
