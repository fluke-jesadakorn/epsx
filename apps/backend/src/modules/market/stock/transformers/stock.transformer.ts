import { StockDocument } from '../types/stock.types';
import { StockResponseDto, PaginatedStockResponse, ScrapingStatusDto } from '../dto/stock.dto';
import type { Paginate } from '@epsx/shared';

export function transformStockToDto(stock: StockDocument): StockResponseDto {
  return {
    _id: stock._id.toString(),
    symbol: stock.symbol,
    company_name: stock.company_name,
    market_cap: stock.market_cap,
    sector: stock.sector,
    volume: stock.volume,
    exchange: stock.exchange.toString(),
    createdAt: stock.createdAt,
    updatedAt: stock.updatedAt
  };
}

export function transformPaginatedResponse(response: Paginate<StockDocument>): PaginatedStockResponse {
  return {
    data: response.data.map(transformStockToDto),
    total: response.total,
    page: response.page || 1,
    limit: response.limit,
    totalPages: Math.ceil(response.total / response.limit)
  };
}

export function transformScrapingResponse(response: ScrapingStatusDto | {
  totalExchanges: number;
  processedExchanges: number;
  totalStocks: number;
  newStocks: number;
  failedExchanges: number;
  errors: string[];
}): ScrapingStatusDto {
  if ('status' in response) {
    return response;
  }
  const { totalExchanges, failedExchanges, errors } = response;
  return {
    status: failedExchanges === 0 ? 'completed' : 'partial',
    processed: totalExchanges - failedExchanges,
    failed: failedExchanges,
    total: totalExchanges,
    error: errors.length > 0 ? errors.join('; ') : undefined
  };
}
