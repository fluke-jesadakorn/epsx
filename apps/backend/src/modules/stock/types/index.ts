import { Model, Document, Types } from 'mongoose';
import { Stock, Exchange } from '@epsx/shared';

// Types for Stock Data
export interface IStockData {
  s: string;  // symbol
  n: string;  // name
}

export interface IStockScreenerResponse {
  data: {
    data: IStockData[];
  };
}

export type IHttpServiceResponse<T> = {
  data: T;
  status: number;
  statusText: string;
  headers: Record<string, string>;
};

export interface IHttpService {
  fetchStockScreener(marketCode: string): Promise<IHttpServiceResponse<IStockScreenerResponse>>;
}

// MongoDB Document Types
export interface IStockDocument extends Document {
  _id: Types.ObjectId;
  symbol: string;
  company_name: string;
  exchange: Types.ObjectId;
}

export interface IStockCreate {
  symbol: string;
  company_name: string;
  exchange: Types.ObjectId;
}

export interface IExchangeDocument extends Document {
  _id: Types.ObjectId;
  market_code: string;
  stocks: Types.ObjectId[];
}

export type IExchangeRef = {
  _id: Types.ObjectId;
  market_code: string;
};

// Service Response Types
export type IScrapingResponse = {
  status: 'COMPLETED' | 'FAILED';
  processed: number;
  failed: number;
  total: number;
  error?: string;
};

// Mongoose Model Types
export type StockModel = Model<IStockDocument>;
export type ExchangeModel = Model<IExchangeDocument>;
