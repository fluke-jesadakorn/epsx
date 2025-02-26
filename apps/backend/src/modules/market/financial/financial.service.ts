import { Injectable, Logger } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Model } from "mongoose";
import { EPSBatchProcessingService } from "./services/eps-batch-processing.service";
import { EpsGrowth, EPSGrowthProcessing, EPSGrowthBatch } from "@epsx/shared";

@Injectable()
export class FinancialService {
  private readonly logger = new Logger(FinancialService.name);

  constructor(
    @InjectModel(EpsGrowth.name)
    private epsGrowthModel: Model<EpsGrowth>,
    @InjectModel(EPSGrowthProcessing.name)
    private epsProcessingModel: Model<EPSGrowthProcessing>,
    @InjectModel(EPSGrowthBatch.name)
    private epsBatchModel: Model<EPSGrowthBatch>,
    private readonly epsBatchProcessingService: EPSBatchProcessingService
  ) {}

  async startEPSGrowthProcessing() {
    this.logger.log("Starting EPS growth processing service");
    const processing = await this.epsBatchProcessingService.startProcessing();
    this.logger.log(`Processing job created with ID: ${processing["_id"]}`);

    // Subscribe to completed batches to store results in EpsGrowth collection
    this.epsBatchProcessingService.onBatchCompleted(async (results) => {
      if (!results?.length) {
        this.logger.warn("Received empty batch results");
        return;
      }

      this.logger.log(`Saving batch results: ${results.length} records`);

      try {
        // Upsert results to EpsGrowth collection
        const operations = await Promise.all(
          results.map(async (result) => {
            const saved = await this.epsGrowthModel.findOneAndUpdate(
              {
                symbol: result.symbol,
                report_date: result.report_date,
                year: result.year,
                quarter: result.quarter,
              },
              result,
              { upsert: true, new: true }
            );
            this.logger.debug(
              `Saved EPS growth data for symbol: ${result.symbol}`
            );
            return saved;
          })
        );

        this.logger.log(
          `Successfully saved ${operations.length} EPS growth records`
        );
      } catch (error) {
        this.logger.error("Error saving batch results:", error);
        throw error;
      }
    });

    return processing;
  }

  async getEPSGrowthProcessingStatus(processingId: string) {
    return this.epsBatchProcessingService.getProcessingStatus(processingId);
  }

  /**
   * Get EPS Growth data from database
   * @param limit Maximum number of results to return
   * @param skip Number of results to skip (for pagination)
   */
  async getEPSGrowthFromDatabase(limit: number = 20, skip: number = 0) {
    try {
      // Get the most recent report date to filter by
      const latestEntry = await this.epsGrowthModel
        .findOne()
        .sort({ report_date: -1, year: -1, quarter: -1 })
        .lean();

      if (!latestEntry) {
        return {
          data: [],
          metadata: {
            skip,
            total: 0,
            page: 1,
            limit,
            totalPages: 0,
          },
        };
      }

      // Query for entries matching the latest report period
      const [data, total] = await Promise.all([
        this.epsGrowthModel
          .find({
            report_date: latestEntry.report_date,
            year: latestEntry.year,
            quarter: latestEntry.quarter,
          })
          .sort({ eps_growth: -1 })
          .skip(skip)
          .limit(limit)
          .lean(),
        this.epsGrowthModel.countDocuments({
          report_date: latestEntry.report_date,
          year: latestEntry.year,
          quarter: latestEntry.quarter,
        }),
      ]);

      return {
        data,
        metadata: {
          skip,
          total,
          page: Math.floor(skip / limit) + 1,
          limit,
          totalPages: Math.ceil(total / limit),
        },
      };
    } catch (error) {
      this.logger.error(
        `Error getting EPS growth from database: ${error}`,
        error
      );
      throw error;
    }
  }

  async getEPSGrowthProcessingStatusBySymbol(
    symbol: string
  ): Promise<EPSGrowthProcessing | null> {
    return this.epsProcessingModel.findOne({ symbol }).exec();
  }

  async getEPSGrowthBatchStatusByMarket(
    marketCode: string
  ): Promise<EPSGrowthBatch | null> {
    return this.epsBatchModel.findOne({ market_code: marketCode }).exec();
  }

  async listEPSGrowthProcessingJobs(
    page = 1,
    limit = 10
  ): Promise<{ data: EPSGrowthProcessing[]; total: number }> {
    const [data, total] = await Promise.all([
      this.epsProcessingModel
        .find()
        .sort({ createdAt: -1 })
        .skip((page - 1) * limit)
        .limit(limit)
        .exec(),
      this.epsProcessingModel.countDocuments(),
    ]);

    return { data, total };
  }

  async listEPSGrowthBatches(
    page = 1,
    limit = 10
  ): Promise<{ data: EPSGrowthBatch[]; total: number }> {
    const [data, total] = await Promise.all([
      this.epsBatchModel
        .find()
        .sort({ requested_at: -1 })
        .skip((page - 1) * limit)
        .limit(limit)
        .exec(),
      this.epsBatchModel.countDocuments(),
    ]);

    return { data, total };
  }
}
