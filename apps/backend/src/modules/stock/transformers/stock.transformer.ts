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

export function transformStockToDto(stock: IStockResponse): StockResponseDto {
  return {
    _id: stock._id,
    symbol: stock.symbol,
    company_name: stock.company_name,
    exchange: stock.exchange,
    createdAt: stock.createdAt,
    updatedAt: stock.updatedAt
  };
}

export function transformPaginatedResponse(response: IPaginatedResponse<IStockResponse>): PaginatedStockResponse {
  return {
    data: response.data.map(transformStockToDto),
    total: response.pagination.total,
    page: response.pagination.page,
    limit: response.pagination.limit,
    totalPages: response.pagination.pages
  };
}

export function transformScrapingResponse(response: IScrapingResponse): ScrapingStatusDto {
  return {
    status: response.status,
    processed: response.processed,
    failed: response.failed,
    total: response.total,
    error: response.error
  };
}
