import { Controller } from "@nestjs/common";
import { MessagePattern, Payload } from "@nestjs/microservices";
import { FinancialService } from "./financial.service";
import {
  AggregationService,
  EpsGrowthData,
} from "./services/aggregation.service";
import { PaginatedResponse } from "./types/financial.type";

interface GetEPSGrowthRankingParams {
  limit?: number;
  skip?: number;
  market_code?: string;
  sortBy?: string;
  sortOrder?: "asc" | "desc";
}

@Controller()
export class FinancialController {
  constructor(
    private readonly financialService: FinancialService,
    private readonly aggregationService: AggregationService
  ) {}

  @MessagePattern({ cmd: "startEPSGrowthProcessing" })
  async startEPSGrowthProcessing() {
    return await this.financialService.startEPSGrowthProcessing();
  }

  @MessagePattern({ cmd: "getEPSGrowthProcessingStatus" })
  async getEPSGrowthProcessingStatus(
    @Payload() data: { processingId: string }
  ) {
    return await this.financialService.getEPSGrowthProcessingStatus(
      data.processingId
    );
  }

  @MessagePattern({ cmd: "calculateAndSaveAllEPSGrowth" })
  async calculateAndSaveAllEPSGrowth() {
    return await this.aggregationService.calculateAndSaveAllEPSGrowth();
  }

  @MessagePattern({ cmd: "getEPSGrowthRanking" })
  async getEPSGrowthRanking(
    @Payload() data: GetEPSGrowthRankingParams
  ): Promise<PaginatedResponse<EpsGrowthData>> {
    const parsedData = {
      limit: data.limit ? parseInt(data.limit.toString(), 10) : 20,
      skip: data.skip ? parseInt(data.skip.toString(), 10) : 0,
      market_code: data.market_code,
      sortBy: data.sortBy || "eps_growth",
      sortOrder: data.sortOrder || "desc",
    };

    return await this.aggregationService.getEPSGrowthRanking(parsedData);
  }

  @MessagePattern({ cmd: "getEPSGrowthRankingOnceQuarter" })
  async getEPSGrowthRankingOnceQuarter(
    @Payload() data: { limit?: number; skip?: number }
  ): Promise<PaginatedResponse<EpsGrowthData>> {
    const parsedLimit = data.limit ? parseInt(data.limit.toString(), 10) : 20;
    const parsedSkip = data.skip ? parseInt(data.skip.toString(), 10) : 0;

    return await this.aggregationService.getEPSGrowthRankingOnceQuarter(
      parsedLimit,
      parsedSkip
    );
  }
}
