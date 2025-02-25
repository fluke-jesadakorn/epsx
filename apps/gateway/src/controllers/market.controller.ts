import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Param,
  Body,
  Query,
  Inject,
  HttpStatus,
} from '@nestjs/common';
import { ClientProxy } from '@nestjs/microservices';
import { firstValueFrom } from 'rxjs';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiParam,
  ApiQuery,
  ApiBody,
} from '@nestjs/swagger';
import {
  CreateExchangeDto,
  ExchangeResponseDto,
  CreateStockDto,
  StockResponseDto,
  FinancialReportDto,
  PaginationQueryDto,
  EPSGrowthResponseDto,
  ScrapeStocksDto,
  ScrapingResponseDto,
  MarketErrorResponseDto,
} from '../swagger/entities/market.swagger';

@ApiTags('Market')
@Controller('market')
export class MarketController {
  constructor(
    @Inject('MARKET_SERVICE')
    private readonly marketService: ClientProxy,
  ) {}

  // Exchange Endpoints
  @Post('exchanges')
  @ApiOperation({ summary: 'Create a new exchange' })
  @ApiBody({ type: CreateExchangeDto })
  @ApiResponse({ 
    status: HttpStatus.CREATED, 
    description: 'Exchange created successfully',
    type: ExchangeResponseDto 
  })
  @ApiResponse({ 
    status: HttpStatus.BAD_REQUEST, 
    description: 'Invalid exchange data',
    type: MarketErrorResponseDto 
  })
  createExchange(@Body() exchangeData: CreateExchangeDto) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'createExchange' }, exchangeData),
    );
  }

  @Get('exchanges')
  @ApiOperation({ summary: 'Get all exchanges with pagination' })
  @ApiQuery({ type: PaginationQueryDto })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'List of exchanges retrieved successfully',
    type: [ExchangeResponseDto] 
  })
  findAllExchanges(
    @Query('skip') skip?: number,
    @Query('limit') limit?: number,
  ) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'findAllExchanges' }, { skip, limit }),
    );
  }

  @Get('exchanges/:marketCode')
  @ApiOperation({ summary: 'Get exchange by market code' })
  @ApiParam({ name: 'marketCode', description: 'Market code of the exchange' })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Exchange found',
    type: ExchangeResponseDto 
  })
  @ApiResponse({ 
    status: HttpStatus.NOT_FOUND, 
    description: 'Exchange not found',
    type: MarketErrorResponseDto 
  })
  findExchange(@Param('marketCode') marketCode: string) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'findExchange' }, { marketCode }),
    );
  }

  @Put('exchanges/:marketCode')
  @ApiOperation({ summary: 'Update exchange by market code' })
  @ApiParam({ name: 'marketCode', description: 'Market code of the exchange' })
  @ApiBody({ type: CreateExchangeDto })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Exchange updated successfully',
    type: ExchangeResponseDto 
  })
  @ApiResponse({ 
    status: HttpStatus.NOT_FOUND, 
    description: 'Exchange not found',
    type: MarketErrorResponseDto 
  })
  updateExchange(
    @Param('marketCode') marketCode: string,
    @Body() updateData: Partial<CreateExchangeDto>,
  ) {
    return firstValueFrom(
      this.marketService.send(
        { cmd: 'updateExchange' },
        { marketCode, updateData },
      ),
    );
  }

  @Delete('exchanges/:marketCode')
  @ApiOperation({ summary: 'Delete exchange by market code' })
  @ApiParam({ name: 'marketCode', description: 'Market code of the exchange' })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Exchange deleted successfully' 
  })
  @ApiResponse({ 
    status: HttpStatus.NOT_FOUND, 
    description: 'Exchange not found',
    type: MarketErrorResponseDto 
  })
  removeExchange(@Param('marketCode') marketCode: string) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'removeExchange' }, { marketCode }),
    );
  }

  // Stock Endpoints
  @Get('stocks')
  @ApiOperation({ summary: 'Get all stocks with pagination' })
  @ApiQuery({ type: PaginationQueryDto })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'List of stocks retrieved successfully',
    type: [StockResponseDto] 
  })
  getAllStocks(@Query('skip') skip?: number, @Query('limit') limit?: number) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'getAllStocks' }, { skip, limit }),
    );
  }

  @Get('stocks/exchange/:exchangeId')
  @ApiOperation({ summary: 'Get stocks by exchange ID' })
  @ApiParam({ name: 'exchangeId', description: 'ID of the exchange' })
  @ApiQuery({ type: PaginationQueryDto })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'List of stocks retrieved successfully',
    type: [StockResponseDto] 
  })
  @ApiResponse({ 
    status: HttpStatus.NOT_FOUND, 
    description: 'Exchange not found',
    type: MarketErrorResponseDto 
  })
  getStocksByExchange(
    @Param('exchangeId') exchangeId: string,
    @Query('skip') skip?: number,
    @Query('limit') limit?: number,
  ) {
    return firstValueFrom(
      this.marketService.send(
        { cmd: 'getStocksByExchange' },
        { exchangeId, skip, limit },
      ),
    );
  }

  @Get('stocks/:symbol')
  @ApiOperation({ summary: 'Get stock by symbol' })
  @ApiParam({ name: 'symbol', description: 'Stock symbol' })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Stock found',
    type: StockResponseDto 
  })
  @ApiResponse({ 
    status: HttpStatus.NOT_FOUND, 
    description: 'Stock not found',
    type: MarketErrorResponseDto 
  })
  getStockBySymbol(@Param('symbol') symbol: string) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'getStockBySymbol' }, { symbol }),
    );
  }

  // Financial Endpoints
  @Get('financials/eps-growth')
  @ApiOperation({ summary: 'Get EPS growth data with pagination' })
  @ApiQuery({ type: PaginationQueryDto })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'EPS growth data retrieved successfully',
    type: [EPSGrowthResponseDto] 
  })
  getEPSGrowth(@Query('limit') limit?: number, @Query('skip') skip?: number) {
    return firstValueFrom(
      this.marketService.send({ cmd: 'getEPSGrowth' }, { limit, skip }),
    );
  }

  // Data Scraping Endpoints
  @Post('scrape/exchanges')
  @ApiOperation({ summary: 'Trigger exchange data scraping' })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Exchange data scraping completed',
    type: ScrapingResponseDto 
  })
  scrapeExchanges() {
    return firstValueFrom(
      this.marketService.send({ cmd: 'scrapeExchanges' }, {}),
    );
  }

  @Post('scrape/stocks')
  @ApiOperation({ summary: 'Trigger stock data scraping for all exchanges' })
  @ApiBody({ type: ScrapeStocksDto, required: false })
  @ApiResponse({ 
    status: HttpStatus.OK, 
    description: 'Stock data scraping for all exchanges completed',
    type: ScrapingResponseDto 
  })
  scrapeStocks() {
    // Send empty object to indicate scraping all exchanges
    return firstValueFrom(
      this.marketService.send({ cmd: 'scrapeStocks' }, {}),
    );
  }
}
