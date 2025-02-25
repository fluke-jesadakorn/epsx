import { Injectable, Logger } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Model, PipelineStage, SortOrder } from "mongoose";
import { EpsGrowth, Financial } from "@app/common/schemas";

import type {
  PaginatedResponse,
  PaginationParams,
  PaginationMetadata,
} from "./types/financial.types";

// Base interface for EPS growth data
interface BaseEpsGrowthData {
  symbol: string;
  company_name: string;
  market_code: string;
}

// EPS Growth specific interfaces
export interface EpsGrowthData extends BaseEpsGrowthData {
  eps_diluted: number;
  previous_eps_diluted: number;
  eps_growth: number;
  report_date: string;
  year: number;
  quarter: number;
}

export interface ThreeQuarterEPSGrowth extends BaseEpsGrowthData {
  quarters: Array<{
    quarter: number;
    year: number;
    eps: number;
    eps_growth: number;
    report_date: string;
  }>;
  average_growth: number;
}

interface BaseGrowthCorrelation extends BaseEpsGrowthData {
  eps_growth: number;
  correlation: number;
  period_start: string;
  period_end: string;
}

export interface EPSPriceGrowth extends BaseGrowthCorrelation {
  price_growth: number;
}

export interface EPSVolumeGrowth extends BaseGrowthCorrelation {
  volume_growth: number;
}

export interface GetEPSGrowthRankingParams extends PaginationParams {
  market_code?: string;
  sortBy: string;
  sortOrder: "asc" | "desc";
}

// MongoDB pipeline stages
const joinStockAndExchange: PipelineStage[] = [
  {
    $lookup: {
      from: "stocks",
      localField: "stock",
      foreignField: "_id",
      as: "stock_info",
    },
  },
  { $unwind: "$stock_info" },
  {
    $lookup: {
      from: "exchanges",
      localField: "stock_info.exchange",
      foreignField: "_id",
      as: "exchange_info",
    },
  },
  { $unwind: "$exchange_info" },
];

const calculatePreviousEps: PipelineStage[] = [
  {
    $setWindowFields: {
      partitionBy: "$stock_info.symbol",
      sortBy: { fiscal_year: -1, fiscal_quarter: -1 },
      output: {
        previous_eps_diluted: {
          $shift: {
            output: "$eps_diluted",
            by: 1,
            default: null,
          },
        },
      },
    },
  },
];

const calculateEpsGrowth: PipelineStage[] = [
  {
    $addFields: {
      eps_growth: {
        $cond: {
          if: {
            $and: [
              { $ne: ["$eps_diluted", null] },
              { $ne: ["$previous_eps_diluted", null] },
              { $ne: ["$previous_eps_diluted", 0] },
            ],
          },
          then: {
            $cond: {
              if: { $gt: ["$previous_eps_diluted", 0] },
              then: {
                $multiply: [
                  {
                    $divide: [
                      { $subtract: ["$eps_diluted", "$previous_eps_diluted"] },
                      "$previous_eps_diluted",
                    ],
                  },
                  100,
                ],
              },
              else: {
                $cond: {
                  if: { $gt: ["$eps_diluted", "$previous_eps_diluted"] },
                  then: 100,
                  else: -100,
                },
              },
            },
          },
          else: 0,
        },
      },
    },
  },
];

@Injectable()
export class AggregationService {
  private readonly logger = new Logger(AggregationService.name);

  constructor(
    @InjectModel(Financial.name)
    private financialModel: Model<Financial>,
    @InjectModel("EpsGrowth")
    private epsGrowthModel: Model<EpsGrowth>
  ) {}

  private calculatePaginationMetadata(
    total: number,
    { limit, skip }: PaginationParams
  ): PaginationMetadata {
    const page = Math.floor(skip / limit) + 1;
    const totalPages = Math.ceil(total / limit);
    return { total, page, limit, totalPages, skip };
  }

  private async saveBatchEpsGrowth(results: any[], batchSize: number = 1000) {
    let processed = 0;
    let failed = 0;

    for (let i = 0; i < results.length; i += batchSize) {
      const batch = results.slice(i, i + batchSize);
      try {
        const operations = batch.map((result) => {
          const { _id, data } = result;
          const { _id: dataId, ...updateData } = data;

          return {
            updateOne: {
              filter: {
                symbol: updateData.symbol,
                year: updateData.year,
                quarter: updateData.quarter,
              },
              update: { $set: updateData },
              upsert: true,
            },
          };
        });

        const bulkResult = await this.epsGrowthModel.bulkWrite(operations);
        processed += bulkResult.modifiedCount + bulkResult.upsertedCount;
        this.logger.log(
          `Processed batch ${i / batchSize + 1}: ${
            bulkResult.modifiedCount + bulkResult.upsertedCount
          } records`
        );
      } catch (error) {
        this.logger.error(
          `Error processing batch ${i / batchSize + 1}:`,
          error
        );
        failed += batch.length;
      }
    }

    return { processed, failed };
  }

  /**
   * Get EPS Growth Ranking for stocks
   */
  /**
   * Get EPS Growth for a single symbol
   */
  async getEPSGrowthForSymbol(symbol: string): Promise<EpsGrowthData | null> {
    this.logger.log(`Calculating EPS growth for symbol: ${symbol}`);
    try {
      if (!symbol) {
        this.logger.warn(
          "Attempting to calculate EPS growth with undefined symbol"
        );
        return null;
      }

      const pipeline: PipelineStage[] = [
        // Match the stock symbol first for better performance
        {
          $lookup: {
            from: "stocks",
            let: { stockId: "$stock" },
            pipeline: [
              {
                $match: {
                  $expr: { $eq: ["$symbol", symbol] },
                },
              },
            ],
            as: "stock_info",
          },
        },
        { $match: { "stock_info.0": { $exists: true } } },
        ...joinStockAndExchange,
        ...calculatePreviousEps,
        ...calculateEpsGrowth,
        { $sort: { fiscal_year: -1, fiscal_quarter: -1 } },
        { $limit: 1 },
        {
          $project: {
            symbol: "$stock_info.symbol",
            company_name: "$stock_info.company_name",
            market_code: "$exchange_info.market_code",
            eps_diluted: 1,
            previous_eps_diluted: 1,
            eps_growth: 1,
            report_date: { $toString: "$report_date" },
            year: "$fiscal_year",
            quarter: "$fiscal_quarter",
          },
        },
      ];

      const results = await this.financialModel.aggregate(pipeline).exec();

      if (!results.length) {
        this.logger.warn(`No financial data found for symbol: ${symbol}`);
        return null;
      }

      this.logger.debug(`Found EPS data for symbol: ${symbol}`, results[0]);
      return results[0];
    } catch (error) {
      this.logger.error(
        `Error calculating EPS growth for symbol ${symbol}:`,
        error
      );
      return null;
    }
  }

  async getEPSGrowthRankingOnceQuarter(
    limit: number = 20,
    skip: number = 0
  ): Promise<PaginatedResponse<EpsGrowthData>> {
    try {
      const pipeline: PipelineStage[] = [
        ...joinStockAndExchange,
        ...calculatePreviousEps,
        ...calculateEpsGrowth,
        {
          $group: {
            _id: "$stock",
            symbol: { $first: "$stock_info.symbol" },
            company_name: { $first: "$stock_info.company_name" },
            market_code: { $first: "$exchange_info.market_code" },
            eps_diluted: { $first: "$eps_diluted" },
            previous_eps_diluted: { $first: "$previous_eps_diluted" },
            eps_growth: { $first: "$eps_growth" },
            report_date: { $first: { $toString: "$report_date" } },
            year: { $first: "$fiscal_year" },
            quarter: { $first: "$fiscal_quarter" },
          },
        },
        { $sort: { year: -1, quarter: -1, eps_growth: -1 } },
        {
          $facet: {
            data: [{ $skip: skip }, { $limit: limit }],
            totalCount: [{ $count: "count" }],
          },
        },
      ];

      const [result] = await this.financialModel.aggregate(pipeline).exec();
      const data = result?.data || [];
      const total = result?.totalCount[0]?.count || 0;

      // Save the results
      const operations = await Promise.all(
        data.map((result) =>
          this.epsGrowthModel.findOneAndUpdate(
            {
              symbol: result.symbol,
              year: result.year,
              quarter: result.quarter,
            },
            result,
            { upsert: true, new: true }
          )
        )
      );

      return {
        data: operations.map((doc) => ({
          ...doc.toObject(),
          _id: doc._id.toString(),
          report_date: doc.report_date.toISOString(),
        })) as EpsGrowthData[],
        metadata: this.calculatePaginationMetadata(total, { limit, skip }),
      };
    } catch (error) {
      this.logger.error("Error calculating EPS growth ranking", error);
      throw error;
    }
  }

  /**
   * Get EPS growth for last 3 quarters
   */
  async getThreeQuarterEPSGrowth(
    limit: number = 20,
    skip: number = 0
  ): Promise<{
    data: ThreeQuarterEPSGrowth[];
    metadata: {
      total: number;
      page: number;
      limit: number;
      totalPages: number;
    };
  }> {
    throw new Error("Method not implemented");
  }

  /**
   * Get correlation between EPS growth and price growth
   */
  async getEPSPriceGrowthCorrelation(
    timeframe: "quarterly" | "annual" = "quarterly",
    limit: number = 20,
    skip: number = 0
  ): Promise<{
    data: EPSPriceGrowth[];
    metadata: {
      total: number;
      page: number;
      limit: number;
      totalPages: number;
    };
  }> {
    throw new Error("Method not implemented");
  }

  /**
   * Calculate and save EPS growth data for all financial records
   */
  async calculateAndSaveAllEPSGrowth() {
    this.logger.log("Starting batch EPS growth calculation and save process");

    try {
      const pipeline: PipelineStage[] = [
        ...joinStockAndExchange,
        ...calculatePreviousEps,
        ...calculateEpsGrowth,
        {
          $project: {
            _id: 0,
            symbol: "$stock_info.symbol",
            company_name: "$stock_info.company_name",
            market_code: "$exchange_info.market_code",
            eps_diluted: 1,
            previous_eps_diluted: 1,
            eps_growth: 1,
            report_date: { $toString: "$report_date" },
            year: "$fiscal_year",
            quarter: "$fiscal_quarter",
          },
        },
        { $sort: { symbol: 1, year: -1, quarter: -1 } },
        {
          $group: {
            _id: "$symbol",
            data: { $first: "$$ROOT" },
          },
        },
      ];

      const results = await this.financialModel.aggregate(pipeline).exec();
      this.logger.log(`Found ${results.length} records to process`);

      const { processed, failed } = await this.saveBatchEpsGrowth(results);

      return {
        status: "completed",
        summary: {
          processed,
          failed,
          total: processed + failed,
        },
      };
    } catch (error) {
      this.logger.error("Error in batch processing:", error);
      throw error;
    }
  }

  /**
   * Get EPS Growth Ranking from pre-calculated data
   */
  async getEPSGrowthRanking(
    params: GetEPSGrowthRankingParams
  ): Promise<PaginatedResponse<EpsGrowthData>> {
    try {
      const query = params.market_code
        ? { market_code: params.market_code }
        : {};
      const sortCriteria: Record<string, SortOrder> = {
        [params.sortBy]: params.sortOrder === "asc" ? 1 : -1,
      };

      const [docs, total] = await Promise.all([
        this.epsGrowthModel
          .find(query)
          .sort(sortCriteria)
          .skip(params.skip)
          .limit(params.limit)
          .lean()
          .exec(),
        this.epsGrowthModel.countDocuments(query),
      ]);

      // Convert dates and ObjectIds to strings
      const data = docs.map((doc) => ({
        ...doc,
        _id: doc._id.toString(),
        report_date: doc.report_date.toISOString(),
      })) as EpsGrowthData[];

      return {
        data,
        metadata: this.calculatePaginationMetadata(total, params),
      };
    } catch (error) {
      this.logger.error("Error fetching EPS growth ranking", error);
      throw error;
    }
  }

  /**
   * Aggregate data for AI-powered graph visualizations
   */
  async getGraphAggregation(params: {
    metrics: string[];
    timeframe: string;
    groupBy?: string;
  }): Promise<any> {
    throw new Error("Method not implemented");
  }
}
