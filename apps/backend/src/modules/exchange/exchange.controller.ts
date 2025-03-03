import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  Query,
  UseGuards,
} from "@nestjs/common";
import { 
  ApiOperation, 
  ApiResponse, 
  ApiTags, 
  ApiParam, 
  ApiQuery,
  ApiBearerAuth,
  ApiSecurity 
} from "@nestjs/swagger";
import { ExchangeService } from "./exchange.service";
import {
  CreateExchangeDto,
  ExchangeResponseDto,
  PaginatedExchangeResponse,
} from "./dto/exchange.dto";

@ApiTags("Exchanges")
@ApiBearerAuth('JWT-auth')
@ApiSecurity('api-key')
@Controller("exchanges")
export class ExchangeController {
  constructor(private readonly exchangeService: ExchangeService) {}

  @Post()
  @ApiOperation({
    summary: "Create a new exchange",
    description: "Creates a new exchange with the provided data",
  })
  @ApiResponse({
    status: 201,
    description: "Exchange successfully created",
    type: ExchangeResponseDto,
  })
  @ApiResponse({
    status: 400,
    description: "Invalid exchange data provided",
  })
  async create(
    @Body() exchangeData: CreateExchangeDto
  ): Promise<ExchangeResponseDto> {
    return this.exchangeService.create(exchangeData);
  }

  @Get()
  @ApiOperation({
    summary: "Get all exchanges",
    description: "Retrieves a paginated list of all exchanges. Results are sorted by market code in ascending order.",
  })
  @ApiQuery({
    name: 'page',
    required: false,
    type: Number,
    description: 'Page number for pagination',
    example: 1,
  })
  @ApiQuery({
    name: 'limit',
    required: false,
    type: Number,
    description: 'Number of items per page',
    example: 10,
  })
  @ApiResponse({
    status: 200,
    description: "List of exchanges successfully retrieved",
    type: PaginatedExchangeResponse,
  })
  @ApiResponse({
    status: 400,
    description: "Invalid query parameters",
  })
  @ApiResponse({
    status: 401,
    description: "Unauthorized - Invalid or missing authentication token",
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - Insufficient permissions",
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
    description: "Retrieves a specific exchange by its market code. The market code is case-sensitive.",
  })
  @ApiParam({
    name: 'marketCode',
    required: true,
    description: 'Market code of the exchange (e.g., NYSE, SET)',
    example: 'NYSE',
  })
  @ApiResponse({
    status: 200,
    description: "Exchange found",
    type: ExchangeResponseDto,
  })
  @ApiResponse({
    status: 401,
    description: "Unauthorized - Invalid or missing authentication token",
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - Insufficient permissions",
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found - The specified market code does not exist",
  })
  async findOne(
    @Param("marketCode") marketCode: string
  ): Promise<ExchangeResponseDto> {
    return this.exchangeService.findOne(marketCode);
  }

  @Put(":marketCode")
  @ApiOperation({
    summary: "Update exchange",
    description: "Updates an exchange by its market code. Only provided fields will be updated.",
  })
  @ApiParam({
    name: 'marketCode',
    required: true,
    description: 'Market code of the exchange to update',
    example: 'NYSE',
  })
  @ApiResponse({
    status: 200,
    description: "Exchange successfully updated",
    type: ExchangeResponseDto,
  })
  @ApiResponse({
    status: 400,
    description: "Invalid request body - Validation failed",
  })
  @ApiResponse({
    status: 401,
    description: "Unauthorized - Invalid or missing authentication token",
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - Insufficient permissions",
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found - The specified market code does not exist",
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
    description: "Permanently deletes an exchange by its market code. This action cannot be undone.",
  })
  @ApiParam({
    name: 'marketCode',
    required: true,
    description: 'Market code of the exchange to delete',
    example: 'NYSE',
  })
  @ApiResponse({
    status: 200,
    description: "Exchange successfully deleted",
  })
  @ApiResponse({
    status: 401,
    description: "Unauthorized - Invalid or missing authentication token",
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - Insufficient permissions",
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found - The specified market code does not exist",
  })
  async remove(@Param("marketCode") marketCode: string): Promise<void> {
    return this.exchangeService.remove(marketCode);
  }

  @Post("scrape")
  @ApiOperation({
    summary: "Scrape exchanges",
    description: "Scrapes exchange data from external sources and saves them to the database. Existing exchanges will be updated with new information.",
  })
  @ApiResponse({
    status: 200,
    description: "Exchanges successfully scraped and saved",
    type: [ExchangeResponseDto],
  })
  @ApiResponse({
    status: 401,
    description: "Unauthorized - Invalid or missing authentication token",
  })
  @ApiResponse({
    status: 403,
    description: "Forbidden - Insufficient permissions",
  })
  @ApiResponse({
    status: 500,
    description: "Error occurred while scraping exchanges - External service may be unavailable",
  })
  async scrapeExchanges(): Promise<ExchangeResponseDto[]> {
    return this.exchangeService.scrapeAndSaveExchanges();
  }
}
