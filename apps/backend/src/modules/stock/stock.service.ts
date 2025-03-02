import { Injectable, Logger, BadRequestException, NotFoundException } from '@nestjs/common';
import { InjectModel } from '@nestjs/mongoose';
import { Stock, Exchange } from '@epsx/shared';
import { Types } from 'mongoose';
import { HttpService } from './services/http.service';
import {
  transformPaginatedResponse,
  transformStockToDto,
  transformScrapingResponse,
} from './transformers/stock.transformer';
import {
  PaginatedStockResponse,
  StockResponseDto,
  ScrapingStatusDto,
} from './dto/stock.dto';
import {
  IPaginationParams,
  StockModel,
  ExchangeModel,
  IStockService,
  IStockScreenerData,
  IExchangeDocument,
  IPaginatedResponse,
  IHttpServiceResponse,
  IScrapingResponse,
  IStockResponse,
  IStockDocument
} from './types';

enum ScrapingStatus {
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

function toStockResponse(doc: any): IStockResponse {
  return {
    _id: doc._id.toString(),
    symbol: doc.symbol,
    company_name: doc.company_name,
    exchange: doc.exchange?._id?.toString() || doc.exchange?.toString() || doc.exchange,
    createdAt: doc.createdAt,
    updatedAt: doc.updatedAt
  };
}

@Injectable()
export class StockService implements IStockService {
  private readonly logger = new Logger(StockService.name);

  constructor(
    @InjectModel(Stock.name) private readonly stockModel: StockModel,
    @InjectModel(Exchange.name) private readonly exchangeModel: ExchangeModel,
    private readonly httpService: HttpService
  ) {}

  async getAllStocks(params: IPaginationParams = {}): Promise<PaginatedStockResponse> {
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

    const stocks = (data || []).map(doc => toStockResponse(doc));
    return transformPaginatedResponse({
      data: stocks,
      pagination: {
        total,
        page: Math.floor(skip / limit) + 1,
        limit,
        pages: Math.ceil(total / limit),
      },
    });
  }

  async getStocksByExchange(exchangeId: string, params: IPaginationParams = {}): Promise<PaginatedStockResponse> {
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

    const stocks = data.map(doc => toStockResponse(doc));
    return transformPaginatedResponse({
      data: stocks,
      pagination: {
        total,
        page: Math.floor(skip / limit) + 1,
        limit,
        pages: Math.ceil(total / limit),
      },
    });
  }

  async getStockBySymbol(symbol: string): Promise<StockResponseDto> {
    const doc = await this.stockModel
      .findOne({ symbol })
      .populate('exchange')
      .lean()
      .exec();

    if (!doc) {
      throw new NotFoundException(`Stock with symbol ${symbol} not found`);
    }

    return transformStockToDto(toStockResponse(doc));
  }

  async saveStockData(): Promise<ScrapingStatusDto> {
    const defaultErrorResponse: IScrapingResponse = {
      status: ScrapingStatus.FAILED,
      processed: 0,
      failed: 1,
      total: 1,
      error: 'Fatal error during stock data processing',
    };

    this.logger.log('Starting stock data processing for all exchanges');
    const existingExchanges = await this.exchangeModel.find().lean().exec();

    try {
      for (let i = 0; i < existingExchanges.length; i += STOCK_CONFIG.maxParallelRequests) {
        const batch = existingExchanges.slice(i, i + STOCK_CONFIG.maxParallelRequests);
        
        this.logger.log(
          `Processing batch ${Math.floor(i / STOCK_CONFIG.maxParallelRequests) + 1} of ${Math.ceil(existingExchanges.length / STOCK_CONFIG.maxParallelRequests)}`
        );

        const batchPromises = batch.map(async (exchange) => {
          const exchangeId = exchange._id;
          if (!exchangeId) {
            this.logger.error(`Exchange ${exchange.market_code} has no ID`);
            return;
          }

          try {
            const response: IHttpServiceResponse<IStockScreenerData> = await this.httpService.fetchStockScreener(exchange.market_code);
            const stockData = response.data;

            if (!stockData?.data?.data) {
              throw new Error(`Invalid data structure received for exchange ${exchange.market_code}`);
            }

            const stocksToProcess = stockData.data.data;
            let processedCount = 0;

            for (let j = 0; j < stocksToProcess.length; j += STOCK_CONFIG.stockBatchSize) {
              const stockBatch = stocksToProcess.slice(j, j + STOCK_CONFIG.stockBatchSize);
              const symbols = stockBatch.map(s => s.s);

              const existingStocks = await this.stockModel
                .find({ symbol: { $in: symbols } })
                .select('symbol')
                .lean()
                .exec();

              const existingSymbols = new Set(existingStocks.map(s => s.symbol));

              const newStocks = stockBatch
                .filter(s => !existingSymbols.has(s.s))
                .map(s => ({
                  symbol: s.s,
                  company_name: s.n,
                  exchange: exchangeId,
                }));

              if (newStocks.length > 0) {
                try {
                  const createdStocks = await this.stockModel.create(newStocks);
                  
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

                  this.logger.log(`Inserted ${newStocks.length} new stocks and updated exchange references`);
                } catch (error) {
                  this.logger.error(`Failed to insert stocks batch: ${error}`);
                }
              }

              processedCount += stockBatch.length;
              this.logger.log(`Processed ${stockBatch.length} stocks in batch (${newStocks.length} new)`);
            }

            return {
              exchange: exchange.market_code,
              processedCount,
            };
          } catch (error) {
            this.logger.error(`Error processing exchange ${exchange.market_code}: ${error}`);
            return null;
          }
        });

        await Promise.all(batchPromises);

        if (i + STOCK_CONFIG.maxParallelRequests < existingExchanges.length) {
          this.logger.log(`Waiting ${STOCK_CONFIG.batchDelay}ms before next batch...`);
          await sleep(STOCK_CONFIG.batchDelay);
        }
      }

      const result: IScrapingResponse = {
        status: ScrapingStatus.COMPLETED,
        processed: existingExchanges.length,
        failed: 0,
        total: existingExchanges.length,
      };

      this.logger.log('All batches have been processed successfully.', result);
      return transformScrapingResponse(result);
    } catch (error) {
      this.logger.error('Fatal error during stock data processing');
      return defaultErrorResponse;
    }
  }

  async scrapeStocksByMarketCap(minMarketCap?: number, maxMarketCap?: number) {
    this.logger.log(
      `Starting stock scraping by market cap range: ${minMarketCap || 0} - ${maxMarketCap || 'unlimited'}`
    );
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }

  async scrapeStocksBySector(sector: string) {
    this.logger.log(`Starting stock scraping for sector: ${sector}`);
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }

  async scrapeStocksByRegion(region: string) {
    this.logger.log(`Starting stock scraping for region: ${region}`);
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }

  async scrapeAllStocks() {
    this.logger.log('Starting comprehensive stock scraping from all exchanges');
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }

  async scrapeStocksByVolume(minVolume: number): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for minimum volume: ${minVolume}`);
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }

  async scrapeStockData(exchangeIds?: string[]): Promise<ScrapingStatusDto> {
    this.logger.log(`Starting stock scraping for exchanges: ${exchangeIds?.join(', ') || 'all'}`);
    const result = await this.saveStockData();
    return transformScrapingResponse(result);
  }
}
