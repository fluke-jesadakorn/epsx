import { Injectable, Logger, BadRequestException, NotFoundException } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Exchange, Stock } from '@epsx/shared';
import { Types } from 'mongoose';
import { HttpService } from './http.service';
import {
  transformPaginatedResponse,
  transformStockToDto,
  transformScrapingResponse,
} from '../transformers/stock.transformer';
import {
  PaginatedStockResponse,
  StockResponseDto,
  ScrapingStatusDto,
} from '../dto/stock.dto';

import {
  IPaginationParams,
  IStockScreenerData,
  IExchangeDocument,
  IPaginatedResponse,
  IHttpServiceResponse,
  IScrapingResponse,
  IStockResponse,
  IStockDocument,
  StockModel,
  ExchangeModel
} from '../types';

export enum ScrapingStatus {
  COMPLETED = 'COMPLETED',
  FAILED = 'FAILED',
}

const STOCK_CONFIG = {
  stockBatchSize: 100,
  maxParallelRequests: 3,
  batchDelay: 1000,
};

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function toStockResponse(doc: Record<string, any>): IStockResponse {
  const _id = doc._id instanceof Types.ObjectId ? doc._id : new Types.ObjectId(doc._id);
  let exchangeId = '';

  if (doc.exchange) {
    if (doc.exchange instanceof Types.ObjectId) {
      exchangeId = doc.exchange.toString();
    } else if (typeof doc.exchange === 'object' && doc.exchange._id) {
      exchangeId = doc.exchange._id.toString();
    } else if (typeof doc.exchange === 'string') {
      exchangeId = doc.exchange;
    }
  }

  return {
    _id: doc._id.toString(),
    symbol: doc.symbol,
    company_name: doc.company_name,
    exchange: exchangeId,
    market_cap: doc.market_cap,
    sector: doc.sector,
    industry: doc.industry,
    website: doc.website,
    description: doc.description,
    ceo: doc.ceo,
    employees: doc.employees,
    headquarters: doc.headquarters,
    createdAt: doc.createdAt,
    updatedAt: doc.updatedAt,
  };
}

@Injectable()
export class StockDataService {
  private readonly logger = new Logger(StockDataService.name);

  constructor(
    @InjectModel(Stock.name) private readonly stockModel: StockModel,
    @InjectModel(Exchange.name) private readonly exchangeModel: ExchangeModel,
    private readonly httpService: HttpService
  ) {}

  async getAllStocks(params: IPaginationParams = {}): Promise<PaginatedStockResponse> {
    try {
      const page = params.page || 1;
      const limit = params.limit || 20;
      const skip = (page - 1) * limit;
      
      const [data, total] = await Promise.all([
        this.stockModel
          .find()
          .populate('exchange')
          .skip(skip)
          .limit(limit)
          .lean()
          .exec(),
        this.stockModel.countDocuments().exec(),
      ]);

      const stocks = (data || []).map((doc: IStockDocument) => toStockResponse(doc));
      return transformPaginatedResponse({
        data: stocks,
        pagination: {
          total,
          page: Math.floor(skip / limit) + 1,
          limit,
          pages: Math.ceil(total / limit),
        },
      });
    } catch (error) {
      this.logger.error('Failed to get all stocks:', error);
      throw error;
    }
  }

  async getStocksByExchange(exchangeId: string, params: IPaginationParams = {}): Promise<PaginatedStockResponse> {
    try {
      if (!exchangeId) {
        throw new BadRequestException('Exchange ID is required');
      }

      const page = params.page || 1;
      const limit = params.limit || 20;
      const skip = (page - 1) * limit;

      const [data, total] = await Promise.all([
        this.stockModel
          .find({ exchange: new Types.ObjectId(exchangeId) })
          .populate('exchange')
          .skip(skip)
          .limit(limit)
          .lean()
          .exec(),
        this.stockModel.countDocuments({ exchange: exchangeId }).exec(),
      ]);

      if (!data?.length) {
        throw new NotFoundException(`No stocks found for exchange ${exchangeId}`);
      }

      const stocks = data.map((doc: IStockDocument) => toStockResponse(doc));
      return transformPaginatedResponse({
        data: stocks,
        pagination: {
          total,
          page: Math.floor(skip / limit) + 1,
          limit,
          pages: Math.ceil(total / limit),
        },
      });
    } catch (error) {
      this.logger.error(`Failed to get stocks for exchange ${exchangeId}:`, error);
      throw error;
    }
  }

  async getStockBySymbol(symbol: string): Promise<StockResponseDto> {
    try {
      const doc = await this.stockModel
        .findOne({ symbol })
        .populate('exchange')
        .lean()
        .exec();

      if (!doc) {
        throw new NotFoundException(`Stock with symbol ${symbol} not found`);
      }

      return transformStockToDto(toStockResponse(doc as IStockDocument));
    } catch (error) {
      this.logger.error(`Failed to get stock with symbol ${symbol}:`, error);
      throw error;
    }
  }

  async saveStockData(): Promise<ScrapingStatusDto> {
    const defaultErrorResponse: IScrapingResponse = {
      status: ScrapingStatus.FAILED,
      processed: 0,
      failed: 1,
      total: 1,
      error: 'Fatal error during stock data processing'
    };

    try {
      this.logger.log('Starting stock data processing for all exchanges');
      const existingExchanges = await this.exchangeModel.find().lean().exec();

      let totalProcessed = 0;
      let totalFailed = 0;

      for (let i = 0; i < existingExchanges.length; i += STOCK_CONFIG.maxParallelRequests) {
        const batch: IExchangeDocument[] = existingExchanges.slice(i, i + STOCK_CONFIG.maxParallelRequests);
        
        this.logger.log(
          `Processing batch ${Math.floor(i / STOCK_CONFIG.maxParallelRequests) + 1} of ${Math.ceil(existingExchanges.length / STOCK_CONFIG.maxParallelRequests)}`
        );

        const batchPromises = batch.map(async (exchange: IExchangeDocument) => {
          const exchangeId = exchange._id;
          if (!exchangeId) {
            this.logger.error(`Exchange ${exchange.market_code} has no ID`);
            return { processed: 0, failed: 1 };
          }

          try {
            const response: IHttpServiceResponse<IStockScreenerData> = await this.httpService.fetchStockScreener(exchange.market_code);
            const stockData = response.data;

            if (!stockData?.data?.data) {
              throw new Error(`Invalid data structure received for exchange ${exchange.market_code}`);
            }

            const stocksToProcess = stockData.data.data;
            let processed = 0;
            let failed = 0;

            for (let j = 0; j < stocksToProcess.length; j += STOCK_CONFIG.stockBatchSize) {
              const stockBatch = stocksToProcess.slice(j, j + STOCK_CONFIG.stockBatchSize);
              const symbols = stockBatch.map((s: { s: string }) => s.s);

              const existingStocks = await this.stockModel
                .find({ symbol: { $in: symbols } })
                .select('symbol')
                .lean()
                .exec();

              const existingSymbols = new Set(existingStocks.map(s => s.symbol));

              const newStocks = stockBatch
                .filter(s => !existingSymbols.has(s.s))
                .map((s: { s: string, n: string }) => ({
                  symbol: s.s,
                  company_name: s.n,
                  exchange: exchangeId,
                }));

              if (newStocks.length > 0) {
                try {
                  const createdStocks = await this.stockModel.create(newStocks) as IStockDocument[];
                  
                  await this.exchangeModel.findByIdAndUpdate(
                    exchangeId,
                    {
                      $push: {
                        stocks: {
                          $each: createdStocks.map(stock => stock._id),
                        },
                      },
                    },
                    { new: true }
                  );

                  processed += newStocks.length;
                  this.logger.log(`Inserted ${newStocks.length} new stocks and updated exchange references`);
                } catch (error) {
                  failed += newStocks.length;
                  this.logger.error(`Failed to insert stocks batch: ${error}`);
                }
              }

              this.logger.log(`Processed ${stockBatch.length} stocks in batch (${newStocks.length} new)`);
            }

            return { processed, failed };
          } catch (error) {
            this.logger.error(`Error processing exchange ${exchange.market_code}: ${error}`);
            return { processed: 0, failed: 1 };
          }
        });

        const results = await Promise.all(batchPromises);
        for (const result of results) {
          if (result) {
            totalProcessed += result.processed;
            totalFailed += result.failed;
          }
        }

        if (i + STOCK_CONFIG.maxParallelRequests < existingExchanges.length) {
          this.logger.log(`Waiting ${STOCK_CONFIG.batchDelay}ms before next batch...`);
          await sleep(STOCK_CONFIG.batchDelay);
        }
      }

      const result: IScrapingResponse = {
        status: ScrapingStatus.COMPLETED,
        processed: totalProcessed,
        failed: totalFailed,
        total: existingExchanges.reduce((acc, exchange) => acc + exchange.stocks.length, 0)
      };

      this.logger.log('All batches have been processed successfully.', result);
      return transformScrapingResponse(result);
    } catch (error) {
      this.logger.error('Fatal error during stock data processing:', error);
      return transformScrapingResponse(defaultErrorResponse);
    }
  }

  async scrapeStocksByMarketCap(minMarketCap?: number, maxMarketCap?: number) {
    this.logger.log(
      `Starting stock scraping by market cap range: ${minMarketCap || 0} - ${maxMarketCap || 'unlimited'}`
    );
    return this.saveStockData();
  }

  async scrapeStocksBySector(sector: string) {
    this.logger.log(`Starting stock scraping for sector: ${sector}`);
    return this.saveStockData();
  }

  async scrapeStocksByRegion(region: string) {
    this.logger.log(`Starting stock scraping for region: ${region}`);
    return this.saveStockData();
  }

  async scrapeAllStocks() {
    this.logger.log('Starting comprehensive stock scraping from all exchanges');
    return this.saveStockData();
  }

  async scrapeStocksByVolume(minVolume: number): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for minimum volume: ${minVolume}`);
    return this.saveStockData();
  }

  async scrapeStockData(exchangeIds?: string[]): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for exchanges: ${exchangeIds?.join(', ') || 'all'}`);
    return this.saveStockData();
  }
}
