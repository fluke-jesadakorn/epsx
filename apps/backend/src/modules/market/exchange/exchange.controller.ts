import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  Query,
} from "@nestjs/common";
import { ApiOperation, ApiResponse, ApiTags } from "@nestjs/swagger";
import { ExchangeService } from "./exchange.service";
import { CreateExchangeDto, ExchangeResponseDto, PaginatedExchangeResponse } from "./dto/exchange.dto";

@ApiTags("Exchanges")
@Controller("exchanges")
export class ExchangeController {
  constructor(private readonly exchangeService: ExchangeService) {}

  @Post()
  @ApiOperation({
    summary: "Create a new exchange",
    description: "Creates a new exchange with the provided data"
  })
  @ApiResponse({
    status: 201,
    description: "Exchange successfully created",
    type: ExchangeResponseDto
  })
  @ApiResponse({
    status: 400,
    description: "Invalid exchange data provided"
  })
  async create(@Body() exchangeData: CreateExchangeDto): Promise<ExchangeResponseDto> {
    return this.exchangeService.create(exchangeData);
  }

  @Get()
  @ApiOperation({
    summary: "Get all exchanges",
    description: "Retrieves a paginated list of all exchanges"
  })
  @ApiResponse({
    status: 200,
    description: "List of exchanges successfully retrieved",
    type: PaginatedExchangeResponse
  })
  async findAll(
    @Query("page") page: number = 1,
    @Query("limit") limit: number = 10
  ): Promise<PaginatedExchangeResponse> {
    const skip = (page - 1) * limit;
    return this.exchangeService.findAll(skip, +limit);
  }

  @Get(":marketCode")
  @ApiOperation({
    summary: "Get exchange by market code",
    description: "Retrieves a specific exchange by its market code"
  })
  @ApiResponse({
    status: 200,
    description: "Exchange found",
    type: ExchangeResponseDto
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found"
  })
  async findOne(@Param("marketCode") marketCode: string): Promise<ExchangeResponseDto> {
    return this.exchangeService.findOne(marketCode);
  }

  @Put(":marketCode")
  @ApiOperation({
    summary: "Update exchange",
    description: "Updates an exchange by its market code"
  })
  @ApiResponse({
    status: 200,
    description: "Exchange successfully updated",
    type: ExchangeResponseDto
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found"
  })
  async update(
    @Param("marketCode") marketCode: string,
    @Body() updateData: CreateExchangeDto
  ): Promise<ExchangeResponseDto> {
    return this.exchangeService.update(marketCode, updateData);
  }

  @Delete(":marketCode")
  @ApiOperation({
    summary: "Delete exchange",
    description: "Deletes an exchange by its market code"
  })
  @ApiResponse({
    status: 200,
    description: "Exchange successfully deleted"
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found"
  })
  async remove(@Param("marketCode") marketCode: string): Promise<void> {
    return this.exchangeService.remove(marketCode);
  }

  @Post("scrape")
  @ApiOperation({
    summary: "Scrape exchanges",
    description: "Scrapes and saves exchange data from external sources"
  })
  @ApiResponse({
    status: 200,
    description: "Exchanges successfully scraped and saved",
    type: [ExchangeResponseDto]
  })
  @ApiResponse({
    status: 500,
    description: "Error occurred while scraping exchanges"
  })
  async scrapeExchanges(): Promise<ExchangeResponseDto[]> {
    return this.exchangeService.scrapeAndSaveExchanges();
  }
}
