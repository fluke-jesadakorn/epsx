import { BaseHttpClient } from '../base/BaseHttpClient';
import type {
  StockItem,
  StockRanking,
  StockFinancialData,
  PortfolioItem,
  WatchlistAddRequest,
  PriceAlert,
  PriceAlertCreateRequest,
  ApiResponse,
  PaginatedResponse,
} from '@epsx/types';

export class AnalyticsClient extends BaseHttpClient {
  // Stock rankings
  async getStockRankings(
    category?: string,
    limit?: number,
    page?: number
  ): Promise<ApiResponse<PaginatedResponse<StockRanking>>> {
    const params = new URLSearchParams();
    if (category) params.append('category', category);
    if (limit) params.append('limit', limit.toString());
    if (page) params.append('page', page.toString());
    
    return this.get<PaginatedResponse<StockRanking>>(`/api/analytics/rankings?${params}`);
  }

  async getStockDetails(symbol: string): Promise<ApiResponse<StockItem>> {
    return this.get<StockItem>(`/api/analytics/stocks/${symbol}`);
  }

  async getStockFinancials(symbol: string): Promise<ApiResponse<StockFinancialData>> {
    return this.get<StockFinancialData>(`/api/analytics/stocks/${symbol}/financials`);
  }

  async searchStocks(query: string, limit?: number): Promise<ApiResponse<StockItem[]>> {
    const params = new URLSearchParams({ query });
    if (limit) params.append('limit', limit.toString());
    
    return this.get<StockItem[]>(`/api/analytics/stocks/search?${params}`);
  }

  // Portfolio
  async getPortfolio(): Promise<ApiResponse<PortfolioItem[]>> {
    return this.get<PortfolioItem[]>('/api/analytics/portfolio');
  }

  async addToPortfolio(symbol: string, shares: number, avgPrice: number): Promise<ApiResponse<PortfolioItem>> {
    return this.post<PortfolioItem>('/api/analytics/portfolio', { symbol, shares, avgPrice });
  }

  async updatePortfolioItem(id: string, shares: number, avgPrice: number): Promise<ApiResponse<PortfolioItem>> {
    return this.put<PortfolioItem>(`/api/analytics/portfolio/${id}`, { shares, avgPrice });
  }

  async removeFromPortfolio(id: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/portfolio/${id}`);
  }

  // Watchlist
  async getWatchlist(): Promise<ApiResponse<StockItem[]>> {
    return this.get<StockItem[]>('/api/analytics/watchlist');
  }

  async addToWatchlist(data: WatchlistAddRequest): Promise<ApiResponse<void>> {
    return this.post<void>('/api/analytics/watchlist', data);
  }

  async removeFromWatchlist(symbol: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/watchlist/${symbol}`);
  }

  // Price alerts
  async getPriceAlerts(): Promise<ApiResponse<PriceAlert[]>> {
    return this.get<PriceAlert[]>('/api/analytics/alerts');
  }

  async createPriceAlert(data: PriceAlertCreateRequest): Promise<ApiResponse<PriceAlert>> {
    return this.post<PriceAlert>('/api/analytics/alerts', data);
  }

  async updatePriceAlert(id: string, data: Partial<PriceAlertCreateRequest>): Promise<ApiResponse<PriceAlert>> {
    return this.put<PriceAlert>(`/api/analytics/alerts/${id}`, data);
  }

  async deletePriceAlert(id: string): Promise<ApiResponse<void>> {
    return this.delete<void>(`/api/analytics/alerts/${id}`);
  }

  async togglePriceAlert(id: string, isActive: boolean): Promise<ApiResponse<PriceAlert>> {
    return this.patch<PriceAlert>(`/api/analytics/alerts/${id}`, { isActive });
  }

  // Market data
  async getMarketOverview(): Promise<ApiResponse<any>> {
    return this.get<any>('/api/analytics/market/overview');
  }

  async getMarketSectors(): Promise<ApiResponse<any[]>> {
    return this.get<any[]>('/api/analytics/market/sectors');
  }

  async getTrendingStocks(limit?: number): Promise<ApiResponse<StockItem[]>> {
    const params = limit ? `?limit=${limit}` : '';
    return this.get<StockItem[]>(`/api/analytics/market/trending${params}`);
  }
}