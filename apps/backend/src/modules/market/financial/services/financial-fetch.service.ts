import { Injectable, Inject, Logger } from "@nestjs/common";
import { HttpService } from "./http.service";
import { InjectModel } from "@nestjs/mongoose";
import { Model } from "mongoose";
import { Financial, Stock } from "@epsx/shared";
import {
  StockWithMarketCode,
  ProcessedFinancialData,
  StockFinancialResponse,
} from "../types/financial.types";
import { processDynamicFinancialData } from "../utils/financial-data.util";
import { Retry, RetryConfig } from "../utils/retry.util";

// Configuration for API retry attempts
const API_RETRY_CONFIG: Partial<RetryConfig> = {
  maxAttempts: 3,
  initialDelay: 1000,
  maxDelay: 15000,
  retryableErrors: [
    "ECONNRESET",
    "ETIMEDOUT",
    "Rate limit exceeded",
    "Too Many Requests",
    /5\d\d/,
    "Failed to fetch",
    "Network Error",
  ],
};

// Configuration for database operations retry
const DB_RETRY_CONFIG: Partial<RetryConfig> = {
  maxAttempts: 3,
  initialDelay: 500,
  maxDelay: 5000,
  retryableErrors: [
    "MongoError",
    "MongoNetworkError",
    "MongoServerError",
    "WriteConflict",
    /Operation .* failed/,
  ],
};

@Injectable()
export class FinancialFetchService {
  private readonly logger = new Logger(FinancialFetchService.name);
  private readonly BATCH_SIZE = 10; // Process 10 stocks at a time
  private readonly DELAY_BETWEEN_REQUESTS = 1000; // 1 second delay between requests

  constructor(
    @InjectModel(Financial.name)
    private financialModel: Model<Financial>,
    @InjectModel(Stock.name)
    private stockModel: Model<Stock>,
    private readonly httpService: HttpService
  ) {}

  /**
   * Checks if a stock with the given symbol already exists in the database
   */
  @Retry(DB_RETRY_CONFIG)
  async checkStockExists(symbol: string): Promise<boolean> {
    const existingStock = await this.stockModel.findOne({ symbol }).exec();
    return !!existingStock;
  }

  /**
   * Fetches financial data for a given stock symbol and processes it
   */
  @Retry(API_RETRY_CONFIG)
  async fetchStockFinancials(
    symbol: string
  ): Promise<ProcessedFinancialData[]> {
    this.logger.log(`Fetching financial data for ${symbol}`);
    const response = await this.httpService.fetchStockAnalysis<any>(
      `/quote/${symbol}/financials/__data.json?p=quarterly&x-sveltekit-trailing-slash=1&x-sveltekit-invalidated=001`
    );

    if (!response) {
      this.logger.warn(`No response data returned for ${symbol}`);
      return [];
    }

    this.logger.debug(
      `Response structure for ${symbol}: ` +
        `hasNodes=${!!response.nodes}, ` +
        `dataType=${typeof response}, ` +
        `keys=[${Object.keys(response).join(", ")}]`
    );

    if (typeof response !== "object") {
      throw new Error(`Invalid response format for ${symbol}: not an object`);
    }

    return processDynamicFinancialData(response);
  }

  /**
   * Validates and normalizes fiscal quarter data
   */
  private validateFiscalQuarter(
    fiscalQuarter: string | number,
    symbol: string
  ): number | null {
    let quarterNumber: number;

    if (typeof fiscalQuarter === "string") {
      quarterNumber = parseInt(fiscalQuarter.replace("Q", ""));
    } else if (typeof fiscalQuarter === "number") {
      quarterNumber = fiscalQuarter;
    } else {
      this.logger.warn(
        `Invalid fiscal quarter type for ${symbol}: ${typeof fiscalQuarter}`
      );
      return null;
    }

    if (quarterNumber < 1 || quarterNumber > 4) {
      this.logger.warn(
        `Invalid fiscal quarter value for ${symbol}: ${quarterNumber}`
      );
      return null;
    }

    return quarterNumber;
  }

  /**
   * Logs financial operation status
   */
  private logFinancialOperation(
    operation: "processing" | "updated" | "inserted" | "completed",
    symbol: string,
    quarter: number,
    year: number
  ): void {
    const periodStr = `${symbol} - Q${quarter} ${year}`;
    switch (operation) {
      case "processing":
        this.logger.log(`Processing financial data for ${periodStr}`);
        break;
      case "updated":
        this.logger.log(`Updated existing financial data for ${periodStr}`);
        break;
      case "inserted":
        this.logger.log(`Inserted new financial data for ${periodStr}`);
        break;
      case "completed":
        this.logger.log(`Successfully saved financial data for ${periodStr}`);
        break;
    }
  }

  /**
   * Saves or updates financial data for a stock
   */
  @Retry(DB_RETRY_CONFIG)
  async saveFinancialData(
    financialData: StockFinancialResponse,
    stock: StockWithMarketCode
  ): Promise<void> {
    if (!financialData.fiscal_quarter) {
      this.logger.warn(`Missing fiscal quarter for ${stock.symbol}`);
      return;
    }

    const quarterNumber = this.validateFiscalQuarter(
      financialData.fiscal_quarter,
      stock.symbol
    );
    if (quarterNumber === null) return;

    const financialRecord = {
      report_date: new Date(financialData.report_date),
      fiscal_quarter: quarterNumber,
      fiscal_year: financialData.fiscal_year,
      revenue: financialData.revenue,
      revenue_growth: financialData.revenue_growth,
      operating_income: financialData.operating_income,
      interest_expense: financialData.interest_expense,
      net_income: financialData.net_income,
      eps_basic: financialData.eps_basic,
      eps_diluted: financialData.eps_diluted,
      free_cash_flow: financialData.free_cash_flow,
      profit_margin: financialData.profit_margin,
      total_operating_expenses: financialData.total_operating_expenses,
    };

    if (!financialRecord.fiscal_quarter || !financialRecord.fiscal_year) {
      this.logger.warn(`Missing fiscal period data for ${stock.symbol}`);
      return;
    }

    this.logFinancialOperation(
      "processing",
      stock.symbol,
      financialRecord.fiscal_quarter,
      financialRecord.fiscal_year
    );

    const existingFinancial = await this.financialModel
      .findOne({
        stock: stock._id,
        fiscal_quarter: financialRecord.fiscal_quarter,
        fiscal_year: financialRecord.fiscal_year,
      })
      .exec();

    if (existingFinancial) {
      await this.financialModel
        .updateOne(
          {
            stock: stock._id,
            fiscal_quarter: financialRecord.fiscal_quarter,
            fiscal_year: financialRecord.fiscal_year,
          },
          { $set: financialRecord }
        )
        .exec();
      this.logFinancialOperation(
        "updated",
        stock.symbol,
        financialRecord.fiscal_quarter,
        financialRecord.fiscal_year
      );
    } else {
      await this.financialModel.create({
        ...financialRecord,
        stock: stock._id,
      });
      this.logFinancialOperation(
        "inserted",
        stock.symbol,
        financialRecord.fiscal_quarter,
        financialRecord.fiscal_year
      );
    }

    this.logFinancialOperation(
      "completed",
      stock.symbol,
      financialRecord.fiscal_quarter,
      financialRecord.fiscal_year
    );
  }
  /**
   * Fetches all stock symbols from the database
   * @returns Promise<string[]> Array of stock symbols
   */
  @Retry(DB_RETRY_CONFIG)
  async getAllStockSymbols(): Promise<string[]> {
    this.logger.log("Fetching all stock symbols from database");
    try {
      const stocks = await this.stockModel.find({}, { symbol: 1 }).exec();

      const symbols = stocks
        .map((stock) => stock.symbol)
        .filter((symbol) => typeof symbol === "string" && symbol.length > 0);

      this.logger.log(`Found ${symbols.length} stock symbols in database`);
      return symbols;
    } catch (error) {
      this.logger.error("Failed to fetch stock symbols from database:", error);
      throw error;
    }
  }

  /**
   * Process and save financial data for multiple stocks
   * @param symbols Array of stock symbols to process
   * @returns Promise<void>
   */
  async processStockBatch(symbols: string[]): Promise<void> {
    this.logger.log(`Processing batch of ${symbols.length} stocks`);

    // Process stocks in smaller batches to avoid overwhelming the API
    for (let i = 0; i < symbols.length; i += this.BATCH_SIZE) {
      const batch = symbols.slice(i, i + this.BATCH_SIZE);

      // Process each stock in the batch concurrently
      await Promise.all(
        batch.map(async (symbol) => {
          try {
            // Check if stock exists in database
            const stockExists = await this.checkStockExists(symbol);
            if (!stockExists) {
              this.logger.warn(
                `Stock ${symbol} not found in database, skipping`
              );
              return;
            }

            // Fetch and process financial data
            const financialData = await this.fetchStockFinancials(symbol);

            // Get stock document with market code and cast to Stock type
            const stockDoc = await this.stockModel
              .findOne({ symbol })
              .populate("exchange")
              .exec();

            if (!stockDoc) {
              this.logger.warn(
                `Stock ${symbol} not found after check, skipping`
              );
              return;
            }

            // Convert Mongoose document to plain object and handle types
            const stockObj = stockDoc.toObject();
            const marketCode =
              stockObj.exchange && typeof stockObj.exchange === "object"
                ? (stockObj.exchange as any).market_code || "UNKNOWN"
                : "UNKNOWN";

            // Get MongoDB _id as string
            const stockId = (stockDoc._id as any).toString();

            // Save each financial data point
            for (const data of financialData) {
              // Convert camelCase to snake_case and handle type conversion
              const financialResponse: StockFinancialResponse = {
                report_date: new Date().toISOString(), // Set current date if not available
                fiscal_quarter: data.fiscalQuarter,
                fiscal_year: data.fiscalYear,
                revenue: data.revenue,
                revenue_growth: data.revenueGrowth,
                operating_income: data.operatingIncome,
                interest_expense: data.interestExpense,
                net_income: data.netIncome,
                eps_basic: data.epsBasic,
                eps_diluted: data.epsDiluted,
                free_cash_flow: data.freeCashFlow,
                profit_margin: data.profitMargin,
                total_operating_expenses: data.totalOperatingExpenses,
                nodes: [], // Required by StockFinancialResponse type
              };

              // Create StockWithMarketCode object
              const stockData: StockWithMarketCode = {
                _id: stockId,
                symbol: stockObj.symbol,
                company_name: stockObj.company_name || null,
                market_code: marketCode,
                exchanges: [
                  {
                    market_code: marketCode,
                    primary: true,
                  },
                ],
              };

              await this.saveFinancialData(financialResponse, stockData);
            }

            this.logger.log(
              `Successfully processed financial data for ${symbol}`
            );
          } catch (error) {
            this.logger.error(`Failed to process stock ${symbol}:`, error);
            // Continue with other stocks even if one fails
          }
        })
      );

      // Add delay between batches to avoid rate limiting
      if (i + this.BATCH_SIZE < symbols.length) {
        await new Promise((resolve) =>
          setTimeout(resolve, this.DELAY_BETWEEN_REQUESTS)
        );
      }
    }
  }

  /**
   * Start the financial data scraping process
   * @returns Promise<void>
   */
  async startFinancialScraping(): Promise<void> {
    this.logger.log("Starting financial data scraping process");

    try {
      // Fetch all stock symbols
      const symbols = await this.getAllStockSymbols();

      // Process all stocks in batches
      await this.processStockBatch(symbols);

      this.logger.log("Financial data scraping process completed");
    } catch (error) {
      this.logger.error("Financial data scraping process failed:", error);
      throw error;
    }
  }
}
