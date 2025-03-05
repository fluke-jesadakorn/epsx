import { Controller, Get, Post, Body, Param, Query } from '@nestjs/common';
import { 
  ApiOperation,
  ApiParam,
  ApiQuery,
  ApiResponse,
  ApiTags
} from '@nestjs/swagger';
import { StockService } from './stock.service';
import { 
  StockPaginationParamsDto,
  StockResponseDto,
  PaginatedStockResponse,
  ScrapeByMarketCapDto,
  ScrapingStatusDto,
} from './dto/stock.dto';

@Controller('stocks')
@ApiTags('Stock')
export class StockController {
  constructor(private readonly stockService: StockService) {}

  @Post()
  @ApiOperation({
    summary: 'Get all stocks',
    description: 'Returns a paginated list of all stocks',
  })
  @ApiResponse({
    status: 200,
    description: "List of stocks successfully retrieved",
    type: PaginatedStockResponse,
  })
  async getAllStocks(
    @Body() params: StockPaginationParamsDto
  ): Promise<PaginatedStockResponse> {
    return this.stockService.getAllStocks(params);
  }

  @Post("exchange/:exchangeId")
  @ApiOperation({
    summary: "Get stocks by exchange",
    description: "Returns a paginated list of stocks for a specific exchange",
  })
  @ApiParam({
    name: "exchangeId",
    description: "ID of the exchange",
    example: "507f1f77bcf86cd799439011",
  })
  @ApiResponse({
    status: 200,
    description: "List of stocks for the exchange retrieved successfully",
    type: PaginatedStockResponse,
  })
  @ApiResponse({
    status: 404,
    description: "Exchange not found",
  })
  async getStocksByExchange(
    @Param('exchangeId') exchangeId: string,
    @Body() params: StockPaginationParamsDto
  ): Promise<PaginatedStockResponse> {
    return this.stockService.getStocksByExchange(exchangeId, params);
  }

  @Get("symbol/:symbol")
  @ApiOperation({
    summary: "Get stock by symbol",
    description: "Returns details for a specific stock symbol",
  })
  @ApiParam({
    name: "symbol",
    description: "Stock symbol",
    example: "AAPL",
  })
  @ApiResponse({
    status: 200,
    description: "Stock details retrieved successfully",
    type: StockResponseDto,
  })
  @ApiResponse({
    status: 404,
    description: "Stock not found",
  })
  async getStockBySymbol(@Param('symbol') symbol: string): Promise<StockResponseDto> {
    return this.stockService.getStockBySymbol(symbol);
  }

  @Get("scrape")
  @ApiOperation({
    summary: "Scrape all stocks",
    description: "Initiates scraping of all stock data",
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeAllStocks(): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeAllStocks();
  }

  @Post("scrape/marketcap")
  @ApiOperation({
    summary: "Scrape stocks by market cap",
    description:
      "Initiates scraping of stocks within specified market cap range",
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping by market cap completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeStocksByMarketCap(
    @Body() data: ScrapeByMarketCapDto
  ): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeStocksByMarketCap(
      data.minMarketCap,
      data.maxMarketCap
    );
  }

  @Get("scrape/sector/:sector")
  @ApiOperation({
    summary: "Scrape stocks by sector",
    description: "Initiates scraping of stocks in a specific sector",
  })
  @ApiParam({
    name: "sector",
    description: "Market sector",
    example: "Technology",
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping by sector completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeStocksBySector(
    @Param('sector') sector: string
  ): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeStocksBySector(sector);
  }

  @Get("scrape/region/:region")
  @ApiOperation({
    summary: "Scrape stocks by region",
    description: "Initiates scraping of stocks in a specific region",
  })
  @ApiParam({
    name: "region",
    description: "Geographic region",
    example: "North America",
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping by region completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeStocksByRegion(
    @Param('region') region: string
  ): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeStocksByRegion(region);
  }

  @Get("scrape/volume/:minVolume")
  @ApiOperation({
    summary: "Scrape stocks by volume",
    description: "Initiates scraping of stocks above a minimum trading volume",
  })
  @ApiParam({
    name: "minVolume",
    description: "Minimum trading volume",
    example: 1000000,
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping by volume completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeStocksByVolume(
    @Param('minVolume') minVolume: number
  ): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeStocksByVolume(minVolume);
  }

  @Get("scrape/exchanges")
  @ApiOperation({
    summary: "Scrape stocks by exchanges",
    description: "Initiates scraping of stocks from specific exchanges",
  })
  @ApiQuery({
    name: "exchangeIds",
    description: "Array of exchange IDs",
    type: [String],
    required: false,
  })
  @ApiResponse({
    status: 200,
    description: "Stock scraping by exchanges completed successfully",
    type: ScrapingStatusDto,
  })
  async scrapeStockData(
    @Query('exchangeIds') exchangeIds?: string[]
  ): Promise<ScrapingStatusDto> {
    return this.stockService.scrapeStockData(exchangeIds);
  }

  @Get("save")
  @ApiOperation({
    summary: "Save stock data",
    description: "Saves scraped stock data to the database",
  })
  @ApiResponse({
    status: 200,
    description: "Stock data saved successfully",
    type: ScrapingStatusDto,
  })
  async saveStockData(): Promise<ScrapingStatusDto> {
    return this.stockService.saveStockData();
  }
}
