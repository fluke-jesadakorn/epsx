import { 
  PaginatedStockResponse, 
  StockResponseDto, 
  ScrapingStatusDto 
} from '../dto/stock.dto';
import { 
  IStockResponse, 
  IPaginatedResponse,
  IScrapingResponse 
} from '../types';
import { ScrapingStatus } from '../services/stock-data.service';

export function transformStockToDto(stock: IStockResponse): StockResponseDto {
  return {
    symbol: stock.symbol,
    name: stock.company_name,
    exchange: stock.exchange,
    marketCap: stock.market_cap || 0,
    sector: stock.sector || '',
    industry: stock.industry || '',
    website: stock.website || '',
    description: stock.description || '',
    ceo: stock.ceo || '',
    employees: stock.employees || 0,
    headquarters: stock.headquarters || ''
  };
}

export function transformPaginatedResponse(response: IPaginatedResponse<IStockResponse>): PaginatedStockResponse {
  return {
    data: response.data.map(transformStockToDto),
    total: response.pagination.total,
    page: response.pagination.page,
    limit: response.pagination.limit
  };
}

export function transformScrapingResponse(response: IScrapingResponse): ScrapingStatusDto {
  const nextTime = new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString();
  
  return {
    status: response.status === ScrapingStatus.COMPLETED ? 'success' : 'failure',
    message: response.error || 'Stock data scraping completed successfully',
    timestamp: new Date().toISOString(),
    totalStocksScraped: response.total,
    duration: response.duration ?? '',
    nextScrape: response.nextScrape ?? nextTime
  };
}
