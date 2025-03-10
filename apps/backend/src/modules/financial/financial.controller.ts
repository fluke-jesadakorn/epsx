import { Controller, Get, Query, Param, Logger } from "@nestjs/common";
import { MessagePattern, Payload } from "@nestjs/microservices";
import { ApiOperation, ApiResponse, ApiTags, ApiQuery } from "@nestjs/swagger";
import { FinancialService } from "./financial.service";
import { AggregationService } from "./services/aggregation.service";
import { FinancialFetchService } from "./services/financial-fetch.service";
import { ProcessingService } from "./services/processing.service";
import {
  ProcessingStatusDto,
  GetEPSGrowthRankingDto,
  PaginatedEpsGrowthResponse,
} from "./dto/financial.dto";
import { 
  EPSGrowthProcessing, 
  EPSGrowthBatch,
  EPSGrowthProcessingDocument,
  EPSGrowthBatchDocument,
  EpsGrowthResponse
} from "@epsx/shared";

@ApiTags("Financial")
@Controller("financial")
export class FinancialController {
  private readonly logger = new Logger(FinancialController.name);

  constructor(
    private readonly financialService: FinancialService,
    private readonly aggregationService: AggregationService,
    private readonly financialFetchService: FinancialFetchService,
    private readonly processingService: ProcessingService
  ) {}

  @MessagePattern({ cmd: "startEPSGrowthProcessing" })
  @ApiOperation({
    summary: "Start EPS growth processing",
    description:
      "Initiates the processing of EPS growth data for all stocks. This process runs asynchronously and returns a processing ID.",
  })
  @ApiResponse({
    status: 200,
    description:
      "Processing started successfully. Returns a processing ID that can be used to check the status.",
    schema: {
      type: 'object',
      example: {
        processingId: '64c9b1b2f1a2b3c4d5e6f7g8',
        status: 'pending',
        message: 'EPS growth processing started successfully'
      },
      properties: {
        processingId: {
          type: 'string',
          description: 'Unique ID for tracking the processing status'
        },
        status: {
          type: 'string',
          description: 'Initial processing status'
        },
        message: {
          type: 'string',
          description: 'Processing status message'
        }
      }
    }
  })
  @ApiResponse({
    status: 500,
    description: "Internal server error occurred while starting the processing",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: { type: "string", example: "Internal server error" },
        error: {
          type: "string",
          example: "Failed to start EPS growth processing",
        },
      },
    },
  })
  async startEPSGrowthProcessing(): Promise<ProcessingStatusDto> {
    return await this.processingService.startEPSGrowthProcessing();
  }

  @MessagePattern({ cmd: "getEPSGrowthProcessingStatus" })
  @ApiOperation({
    summary: "Get EPS growth processing status",
    description:
      "Returns the current status of EPS growth data processing for a specific processing ID",
  })
  @ApiResponse({
    status: 200,
    description: "Processing status retrieved successfully",
    type: ProcessingStatusDto,
  })
  @ApiResponse({
    status: 404,
    description: "Processing ID not found",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 404 },
        message: { type: "string", example: "Processing ID not found" },
      },
    },
  })
  @ApiResponse({
    status: 500,
    description: "Internal server error occurred while fetching the status",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: { type: "string", example: "Internal server error" },
      },
    },
  })
  async getEPSGrowthProcessingStatus(
    @Payload() data: { processingId: string }
  ): Promise<ProcessingStatusDto> {
    return await this.processingService.getEPSGrowthProcessingStatus(
      data.processingId
    );
  }

  @MessagePattern({ cmd: "calculateAndSaveAllEPSGrowth" })
  @ApiOperation({
    summary: "Calculate and save EPS growth",
    description:
      "Calculates and saves EPS growth data for all stocks. This operation processes and updates the EPS growth rankings in the database.",
  })
  @ApiResponse({
    status: 200,
    description: "EPS growth calculation completed successfully",
    type: ProcessingStatusDto,
  })
  @ApiResponse({
    status: 500,
    description: "Internal server error occurred during calculation",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: { type: "string", example: "Failed to calculate EPS growth" },
      },
    },
  })
  async calculateAndSaveAllEPSGrowth() {
    return await this.aggregationService.calculateAndSaveAllEPSGrowth();
  }

  @MessagePattern({ cmd: "getEPSGrowthRanking" })
  @ApiOperation({
    summary: "Get EPS growth ranking",
    description:
      "Returns a paginated list of stocks ranked by EPS growth. Results can be filtered by market code and sorted by specified fields.",
  })
  @ApiResponse({
    status: 200,
    description: "EPS growth ranking retrieved successfully",
    schema: {
      type: 'object',
      example: {
        data: [
          {
            symbol: 'AAPL',
            companyName: 'Apple Inc.',
            epsGrowth: 0.15,
            marketCode: 'NASDAQ'
          },
          {
            symbol: 'MSFT',
            companyName: 'Microsoft Corporation',
            epsGrowth: 0.12,
            marketCode: 'NASDAQ'
          }
        ],
        total: 100,
        page: 1,
        limit: 20
      },
      properties: {
        data: {
          type: 'array',
          items: {
            type: 'object',
            properties: {
              symbol: { type: 'string' },
              companyName: { type: 'string' },
              epsGrowth: { type: 'number' },
              marketCode: { type: 'string' }
            }
          }
        },
        total: { type: 'number' },
        page: { type: 'number' },
        limit: { type: 'number' }
      }
    }
  })
  @ApiResponse({
    status: 400,
    description: "Invalid request parameters",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 400 },
        message: { type: "string", example: "Invalid parameters" },
        errors: {
          type: "array",
          items: {
            type: "string",
          },
          example: ["Invalid sort order", "Invalid limit value"],
        },
      },
    },
  })
  @ApiResponse({
    status: 500,
    description: "Internal server error occurred while retrieving rankings",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: {
          type: "string",
          example: "Failed to retrieve EPS growth ranking",
        },
      },
    },
  })
  async getEPSGrowthRanking(
    @Payload() data: GetEPSGrowthRankingDto
  ): Promise<PaginatedEpsGrowthResponse> {
    const parsedData = {
      limit: data.limit ? parseInt(String(data.limit), 10) : 20,
      skip: data.skip ? parseInt(String(data.skip), 10) : 0,
      market_code: data.market_code,
      sortBy: data.sortBy || "eps_growth",
      sortOrder: data.sortOrder || "desc",
    };

    return await this.aggregationService.getEPSGrowthRanking(parsedData);
  }

  @MessagePattern({ cmd: "getEPSGrowthRankingOnceQuarter" })
  @ApiOperation({
    summary: "Get EPS growth ranking for one quarter",
    description:
      "Returns a paginated list of stocks ranked by EPS growth for a single quarter. This endpoint returns data for the most recent quarter available in the database.",
  })
  @ApiResponse({
    status: 200,
    description: "Single quarter EPS growth ranking retrieved successfully",
    schema: {
      type: "object",
      properties: {
        data: {
          type: "array",
          items: {
            $ref: "#/components/schemas/EpsGrowthResponse"
          }
        },
        total: { type: "number" },
        page: { type: "number" },
        limit: { type: "number" }
      }
    },
  })
  @ApiResponse({
    status: 400,
    description: "Invalid pagination parameters",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 400 },
        message: { type: "string", example: "Invalid pagination parameters" },
        errors: {
          type: "array",
          items: {
            type: "string",
          },
          example: [
            "Limit must be a positive number",
            "Skip must be a non-negative number",
          ],
        },
      },
    },
  })
  @ApiResponse({
    status: 404,
    description: "No data available for any quarter",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 404 },
        message: { type: "string", example: "No EPS growth data available" },
      },
    },
  })
  @ApiResponse({
    status: 500,
    description: "Internal server error occurred while retrieving quarter data",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: {
          type: "string",
          example: "Failed to retrieve quarter EPS growth ranking",
        },
      },
    },
  })
  async getEPSGrowthRankingOnceQuarter(
    @Payload() data: { limit?: number; skip?: number }
  ): Promise<PaginatedEpsGrowthResponse> {
    const parsedLimit = data.limit ? parseInt(data.limit.toString(), 10) : 20;
    const parsedSkip = data.skip ? parseInt(data.skip.toString(), 10) : 0;

    return await this.aggregationService.getEPSGrowthRankingOnceQuarter(
      parsedLimit,
      parsedSkip
    );
  }

  // REST API endpoints with Swagger documentation
  @Get("eps/processing/:symbol")
  @ApiOperation({ summary: "Get EPS processing status for a symbol" })
  @ApiResponse({
    status: 200,
    description: "Returns the EPS processing status",
    type: EPSGrowthProcessing,
  })
  @ApiResponse({ status: 404, description: "Processing status not found" })
  async getEpsProcessingStatus(
    @Param("symbol") symbol: string
  ): Promise<EPSGrowthProcessingDocument | null> {
    return this.processingService.getEPSGrowthProcessingStatusBySymbol(symbol);
  }

  @Get("eps/batch/:marketCode")
  @ApiOperation({ summary: "Get EPS batch processing status for a market" })
  @ApiResponse({
    status: 200,
    description: "Returns the batch processing status",
    type: EPSGrowthBatch,
  })
  @ApiResponse({ status: 404, description: "Batch status not found" })
  async getEpsBatchStatus(
    @Param("marketCode") marketCode: string
  ): Promise<EPSGrowthBatchDocument | null> {
    return this.processingService.getEPSGrowthBatchStatusByMarket(marketCode);
  }

  @Get("eps/processing")
  @ApiOperation({ summary: "List all EPS processing jobs" })
  @ApiQuery({
    name: "page",
    required: false,
    type: Number,
    description: "Page number (default: 1)",
    example: 1
  })
  @ApiQuery({
    name: "limit",
    required: false,
    type: Number,
    description: "Items per page (default: 10)",
    example: 10
  })
  @ApiResponse({
    status: 200,
    description: "Returns a paginated list of processing jobs",
    schema: {
      properties: {
        data: {
          type: "array",
          items: { $ref: "#/components/schemas/EPSGrowthProcessing" },
        },
        total: { type: "number" },
      },
    },
  })
  async listEpsProcessingJobs(
    @Query("page") page = 1,
    @Query("limit") limit = 10
  ) {
    return this.processingService.listEPSGrowthProcessingJobs(+page, +limit);
  }

  @Get("eps/batches")
  @ApiOperation({ summary: "List all EPS batch processing jobs" })
  @ApiQuery({
    name: "page",
    required: false,
    type: Number,
    description: "Page number (default: 1)",
  })
  @ApiQuery({
    name: "limit",
    required: false,
    type: Number,
    description: "Items per page (default: 10)",
  })
  @ApiResponse({
    status: 200,
    description: "Returns a paginated list of batch processing jobs",
    schema: {
      properties: {
        data: {
          type: "array",
          items: { $ref: "#/components/schemas/EPSGrowthBatch" },
        },
        total: { type: "number" },
      },
    },
  })
  async listEpsBatches(@Query("page") page = 1, @Query("limit") limit = 10) {
    return this.processingService.listEPSGrowthBatches(+page, +limit);
  }

  @Get("scrape")
  @ApiOperation({
    summary: "Start financial data scraping",
    description:
      "Initiates the scraping process to fetch and store financial data for all stocks from stockanalysis.com. Returns success status when initiated.",
  })
  @ApiResponse({
    status: 200,
    description: "Financial data scraping process started successfully",
    schema: {
      type: "object",
      example: {
        status: "success",
        message: "Financial data scraping process started",
        timestamp: "2025-03-06T10:40:09Z",
        estimatedDuration: "2 hours",
        progressUrl: "/financial/scrape/progress/64c9b1b2f1a2b3c4d5e6f7g8"
      },
      properties: {
        status: { type: "string" },
        message: { type: "string" },
        timestamp: { type: "string" },
        estimatedDuration: { type: "string" },
        progressUrl: { type: "string" }
      }
    },
  })
  @ApiResponse({
    status: 500,
    description:
      "Internal server error occurred while starting the scraping process",
    schema: {
      type: "object",
      properties: {
        statusCode: { type: "number", example: 500 },
        message: {
          type: "string",
          example: "Failed to start scraping process",
        },
        error: { type: "string", example: "Internal server error" },
      },
    },
  })
  async startScraping() {
    try {
      await this.financialFetchService.startFinancialScraping();
      return {
        status: "success",
        message: "Financial data scraping process started",
      };
    } catch (error) {
      this.logger.error("Failed to start financial data scraping:", error);
      throw new Error("Failed to start financial data scraping process");
    }
  }
}
