import { Injectable, Logger } from "@nestjs/common";
import {
  StockWithMarketCode,
  WorkerConfig,
  StockFinancialResponse,
} from "../types/financial.types";
import { FinancialFetchService } from "./financial-fetch.service";
import { FetchStateService } from "./fetch-state.service";

interface RetryConfig {
  maxRetries: number;
  initialDelay: number;
  maxDelay: number;
  backoffFactor: number;
}

const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 3,
  initialDelay: 1000, // 1 second
  maxDelay: 10000, // 10 seconds
  backoffFactor: 2,
};

@Injectable()
export class WorkerPoolService {
  private readonly logger = new Logger(WorkerPoolService.name);

  constructor(
    private readonly financialFetch: FinancialFetchService,
    private readonly fetchState: FetchStateService
  ) {}

  private async retryWithBackoff<T>(
    operation: () => Promise<T>,
    stock: StockWithMarketCode,
    retryConfig: RetryConfig = DEFAULT_RETRY_CONFIG
  ): Promise<T> {
    let lastError: Error = new Error("No attempts made");
    let delay = retryConfig.initialDelay;

    for (let attempt = 0; attempt <= retryConfig.maxRetries; attempt++) {
      try {
        if (attempt > 0) {
          this.logger.log(
            `Retry attempt ${attempt} for ${stock.symbol} after ${delay}ms delay`
          );
          await WorkerPoolService.sleep(delay);
        }
        return await operation();
      } catch (e) {
        const error = e as Error;
        lastError = error;

        // Check for duplicate key error
        if (error.message?.includes("E11000 duplicate key error")) {
          this.logger.log(
            `Stock ${stock.symbol} already exists for market ${stock.market_code}, skipping`
          );
          return undefined as T;
        }

        this.logger.warn(
          `Attempt ${attempt + 1}/${retryConfig.maxRetries + 1} failed for ${
            stock.symbol
          }: ${error.message}`
        );

        // Determine if we should retry based on error type
        if (this.shouldRetry(error as any)) {
          // Calculate next delay with exponential backoff
          delay = Math.min(
            delay * retryConfig.backoffFactor,
            retryConfig.maxDelay
          );
        } else {
          // If error is not retryable, throw error;
          throw error;
        }
      }
    }

    throw new Error(
      `Failed to process ${stock.symbol} after ${
        retryConfig.maxRetries + 1
      } attempts. Last error: ${lastError.message}`
    );
  }

  private shouldRetry(error: any): boolean {
    // Don't retry on duplicate key errors
    if (error.message?.includes("E11000 duplicate key error")) {
      return false;
    }

    // Retry on network errors or rate limiting
    if (error.response) {
      const status = error.response.status;
      // Retry on rate limiting (429) or server errors (500-599)
      return status === 429 || (status >= 500 && status < 600);
    }
    // Retry on network errors (no response)
    return !error.response && error.request;
  }

  async processStocksBatch(
    stocks: StockWithMarketCode[],
    lastProcessedStock: string | null,
    config: WorkerConfig
  ): Promise<number> {
    let index = lastProcessedStock
      ? stocks.findIndex((stock) => stock.symbol === lastProcessedStock) + 1
      : 0;

    let processedCount = 0;
    const errors: Array<{ symbol: string; error: string }> = [];
    const processedSymbols = new Set<string>();

    const processStock = async (stock: StockWithMarketCode) => {
      // Skip if we've already processed this symbol in this batch
      const symbolKey = `${stock.symbol}_${stock.market_code}`;
      if (processedSymbols.has(symbolKey)) {
        this.logger.log(
          `Skipping duplicate stock ${stock.symbol} for market ${stock.market_code}`
        );
        return;
      }

      try {
        await this.retryWithBackoff(async () => {
          const processedData = await this.financialFetch.fetchStockFinancials(
            stock.symbol
          );
          if (processedData && processedData.length > 0) {
            const latestData = processedData[0];
            // Validate the financial data before proceeding
            if (!latestData) {
              throw new Error(
                `No valid financial data found for ${stock.symbol}`
              );
            }

            // Create financial data with proper structure matching StockFinancialResponse
            const financialData: StockFinancialResponse = {
              nodes: [{ data: [latestData] }], // Wrap latestData in array as per interface
              report_date: new Date().toISOString(),
              fiscal_quarter: latestData.fiscalQuarter ?? undefined,
              fiscal_year: latestData.fiscalYear ?? undefined,
              revenue: latestData.revenueGrowth ?? undefined,
              operating_income: latestData.operatingIncome ?? undefined,
              interest_expense: latestData.interestExpense ?? undefined,
              net_income: latestData.netIncome ?? undefined,
              eps_basic: latestData.epsBasic ?? undefined,
              eps_diluted: latestData.epsDiluted ?? undefined,
              free_cash_flow: latestData.freeCashFlow ?? undefined,
              profit_margin: latestData.profitMargin ?? undefined,
              total_operating_expenses:
                latestData.totalOperatingExpenses ?? undefined,
            };

            // Enhanced validation before saving
            if (!financialData.fiscal_quarter || !financialData.fiscal_year) {
              this.logger.warn(
                `Missing required fiscal data for ${stock.symbol}, skipping save`
              );
              return;
            }

            // Log data validation for debugging
            this.logger.debug(
              `Validated financial data for ${stock.symbol}: ` +
                `Q${financialData.fiscal_quarter || "N/A"} ` +
                `FY${financialData.fiscal_year || "N/A"}`
            );

            await this.financialFetch.saveFinancialData(financialData, stock);
            processedCount++;
            // Mark this symbol as processed
            processedSymbols.add(symbolKey);
            this.logger.log(`Successfully processed ${stock.symbol}`);
          }
        }, stock);
      } catch (e) {
        const error = e as Error;
        // Only log non-duplicate errors as actual errors
        if (!error.message?.includes("E11000 duplicate key error")) {
          errors.push({
            symbol: stock.symbol,
            error: error.message,
          });
          this.logger.error(`Failed to process ${stock.symbol}`, error);
        } else {
          // Log duplicate as info
          this.logger.log(
            `Skipped duplicate stock ${stock.symbol} for market ${stock.market_code}`
          );
        }
      }
    };

    // Enhanced concurrent processing with semaphore-like control
    // Queue-based processing implementation with memory management
    const queue: StockWithMarketCode[] = stocks.slice(index);
    const activeWorkers = new Set<Promise<void>>();
    const CHUNK_SIZE = 50; // Process stocks in smaller chunks to manage memory

    const worker = async () => {
      while (queue.length > 0) {
        // Process in chunks to prevent memory buildup
        const chunk = queue.splice(0, CHUNK_SIZE);

        for (const stock of chunk) {
          // Wait if we've reached max concurrent requests
          while (activeWorkers.size >= config.maxConcurrentRequests) {
            await Promise.race(Array.from(activeWorkers));
          }

          // Clean up completed workers
          for (const worker of activeWorkers) {
            // Remove completed promises
            Promise.race([worker, Promise.resolve()]).then(() => {
              activeWorkers.delete(worker);
            });
          }

          // Create worker promise with cleanup
          const workerPromise = processStock(stock).finally(() => {
            activeWorkers.delete(workerPromise);
          });

          // Add to active workers set
          activeWorkers.add(workerPromise);
        }

        // Allow GC to clean up processed chunk
        global.gc?.();
      }

      // Wait for remaining workers in chunk to complete
      if (activeWorkers.size > 0) {
        await Promise.all(Array.from(activeWorkers));
        activeWorkers.clear();
      }
    };

    // Start workers
    await Promise.all(
      Array(config.maxConcurrentRequests)
        .fill(null)
        .map(() => worker())
    );

    // Log summary of processing
    if (errors.length > 0) {
      this.logger.warn(
        `Processing completed with ${errors.length} errors:\n${JSON.stringify(errors, null, 2)}`
      );
    }

    return processedCount;
  }

  static async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
