import { Injectable, Logger } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import { Model } from "mongoose";
import { EventEmitter } from "events";
import {
  EPSGrowthProcessing,
  EPSGrowthBatch,
  Financial,
  Stock,
} from "@app/common/schemas";
import { AggregationService } from "../aggregation.service";

@Injectable()
export class EPSBatchProcessingService extends EventEmitter {
  private readonly logger = new Logger(EPSBatchProcessingService.name);
  private readonly BATCH_SIZE = 50; // Process 50 stocks at a time
  private static readonly BATCH_COMPLETED_EVENT = "batchCompleted";

  constructor(
    @InjectModel(EPSGrowthProcessing.name)
    private epsProcessingModel: Model<EPSGrowthProcessing>,
    @InjectModel(EPSGrowthBatch.name)
    private epsBatchModel: Model<EPSGrowthBatch>,
    @InjectModel(Stock.name)
    private stockModel: Model<Stock>,
    private readonly aggregationService: AggregationService
  ) {
    super();
  }

  async startProcessing(): Promise<EPSGrowthProcessing> {
    this.logger.log("Starting EPS growth batch processing");

    // Check if there's already a processing job running
    const existingJob = await this.epsProcessingModel.findOne({
      status: "processing",
    });
    if (existingJob) {
      this.logger.log(`Found existing processing job: ${existingJob._id}`);
      return existingJob;
    }

    // Create new processing job
    const totalStocks = await this.stockModel.countDocuments();
    this.logger.log(`Found ${totalStocks} stocks to process`);

    const processing = await this.epsProcessingModel.create({
      totalStocks,
      status: "processing",
    });
    this.logger.log(`Created new processing job: ${processing._id}`);

    // Create batches
    const stocks = await this.stockModel.find().select("symbol").lean();
    const batches = [];

    for (let i = 0; i < stocks.length; i += this.BATCH_SIZE) {
      const batchSymbols = stocks
        .slice(i, i + this.BATCH_SIZE)
        .map((s) => s.symbol);
      batches.push({
        processingId: processing._id,
        batchNumber: Math.floor(i / this.BATCH_SIZE),
        symbols: batchSymbols,
      });
    }

    await this.epsBatchModel.insertMany(batches);
    this.logger.log(`Created ${batches.length} batches for processing`);

    // Start processing first batch
    setImmediate(() => {
      this.processBatch(processing._id.toString(), 0).catch((err) =>
        this.logger.error(`Error processing batch 0: ${err.message}`, err.stack)
      );
    });

    this.logger.log("Batch processing initiated");
    return processing;
  }

  private async processBatch(
    processingId: string,
    batchNumber: number
  ): Promise<void> {
    const batch = await this.epsBatchModel.findOne({
      processingId,
      batchNumber,
    });

    if (!batch || batch.isProcessed) {
      return;
    }

    try {
      batch.status = "processing";
      await batch.save();

      // Process each symbol in batch
      this.logger.log(
        `Processing batch ${batchNumber} with ${batch.symbols.length} symbols`
      );
      const results = [];
      for (const symbol of batch.symbols) {
        this.logger.debug(`Processing symbol: ${symbol}`);
        const result =
          await this.aggregationService.getEPSGrowthForSymbol(symbol);
        if (result) {
          results.push(result);
          this.logger.debug(`Successfully processed symbol: ${symbol}`);
        } else {
          this.logger.warn(`No results found for symbol: ${symbol}`);
        }
      }

      try {
        // Filter out null results and validate data
        const validResults = results.filter(
          (result) =>
            result &&
            result.symbol &&
            result.company_name &&
            result.market_code &&
            typeof result.eps_diluted === "number" &&
            typeof result.previous_eps_diluted === "number" &&
            typeof result.eps_growth === "number" &&
            result.report_date &&
            result.year &&
            result.quarter
        );

        if (validResults.length > 0) {
          // Update batch with results and emit event
          batch.results = validResults;
          batch.isProcessed = true;
          batch.status = "completed";
          await batch.save();

          // Emit batch completion event with results
          this.emit(
            EPSBatchProcessingService.BATCH_COMPLETED_EVENT,
            validResults
          );

          // Update processing status
          const processing =
            await this.epsProcessingModel.findById(processingId);
          if (processing) {
            processing.processedStocks =
              (processing.processedStocks || 0) + batch.symbols.length;
            processing.lastProcessedSymbol =
              batch.symbols[batch.symbols.length - 1];

            if (processing.processedStocks >= processing.totalStocks) {
              processing.status = "completed";
              processing.isCompleted = true;
              processing.completedTime = new Date();
            }

            await processing.save();
          }
        } else {
          this.logger.warn(`No valid results found for batch ${batchNumber}`);
          batch.status = "completed";
          batch.isProcessed = true;
          await batch.save();
        }
      } catch (error) {
        this.logger.error(
          `Error processing batch results ${batchNumber}: ${error.message}`,
          error.stack
        );
        throw error;
      }

      // Get the latest processing status
      const latestProcessing =
        await this.epsProcessingModel.findById(processingId);

      // Process next batch if not complete
      if (latestProcessing && !latestProcessing.isCompleted) {
        this.processBatch(processingId, batchNumber + 1).catch((err) =>
          this.logger.error(
            `Error processing batch ${batchNumber + 1}: ${err.message}`,
            err.stack
          )
        );
      }
    } catch (error) {
      this.logger.error(
        `Error processing batch ${batchNumber}: ${error.message}`,
        error.stack
      );

      batch.status = "error";
      batch.error = error.message;
      await batch.save();

      // Mark processing as error
      await this.epsProcessingModel.findByIdAndUpdate(processingId, {
        status: "error",
        error: `Error in batch ${batchNumber}: ${error.message}`,
      });
    }
  }

  async getProcessingStatus(
    processingId: string
  ): Promise<EPSGrowthProcessing> {
    return this.epsProcessingModel.findById(processingId);
  }

  onBatchCompleted(listener: (results: any[]) => void) {
    this.on(EPSBatchProcessingService.BATCH_COMPLETED_EVENT, listener);
  }
}
