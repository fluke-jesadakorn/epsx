import { Document, Model, Types } from 'mongoose';

export interface IPaginationParams {
  page?: number;
  limit?: number;
}

export interface IPaginationResponse {
  total: number;
  page: number;
  limit: number;
  pages: number;
}

export interface IHttpResponse<T = any> {
  data: {
    data: T[];
  };
}

export interface IHttpServiceResponse<T = any> {
  data: IHttpResponse<T>;
  status: number;
  statusText: string;
  headers: Record<string, string>;
}

export interface ITimestamps {
  createdAt: Date;
  updatedAt: Date;
}

export interface IExchangeRef {
  _id: Types.ObjectId;
  market_code: string;
}

// Base interfaces without Mongoose specifics
export interface IStockBase {
  symbol: string;
  company_name: string;
  exchange: Types.ObjectId | IExchangeRef;
}

export interface IExchangeBase {
  market_code: string;
  stocks: Types.ObjectId[];
}

// Mongoose document interfaces
export interface IStockDocument extends Document {
  symbol: string;
  company_name: string;
  exchange: Types.ObjectId | IExchangeDocument;
  createdAt: Date;
  updatedAt: Date;
}

export interface IExchangeDocument extends Document {
  market_code: string;
  stocks: Types.ObjectId[];
  createdAt: Date;
  updatedAt: Date;
}

// Model types
export type StockModel = Model<IStockDocument>;
export type ExchangeModel = Model<IExchangeDocument>;

// API types
export interface IStockScreenerData {
  s: string;  // symbol
  n: string;  // name
}

export interface IStockResponse {
  _id: string;
  symbol: string;
  company_name: string;
  exchange: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface IPaginatedResponse<T> {
  data: T[];
  pagination: IPaginationResponse;
}

export interface IScrapingResponse {
  status: string;
  processed: number;
  failed: number;
  total: number;
  error?: string;
}

import { 
  PaginatedStockResponse,
  StockResponseDto,
  ScrapingStatusDto
} from './dto/stock.dto';

export interface IStockService {
  getAllStocks(params?: IPaginationParams): Promise<PaginatedStockResponse>;
  getStocksByExchange(exchangeId: string, params?: IPaginationParams): Promise<PaginatedStockResponse>;
  getStockBySymbol(symbol: string): Promise<StockResponseDto>;
  saveStockData(): Promise<ScrapingStatusDto>;
}

// Utility types for type assertions
export type MongooseDoc<T> = T & Document;

export interface IHttpService {
  fetchStockScreener(marketCode: string): Promise<IHttpServiceResponse<IStockScreenerData>>;
}
