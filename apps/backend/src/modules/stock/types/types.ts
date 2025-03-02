import { Model } from "mongoose";

export interface PaginationParams {
  page?: number;
  limit?: number;
}

export interface IStockModel {
  symbol: string;
  company_name: string;
  exchange: string;
  _id?: string;
}

export interface IExchangeModel {
  market_code: string;
  stocks: string[];
  _id?: string;
}

export interface IStockService {
  getAllStocks(params?: PaginationParams): Promise<any>;
  getStocksByExchange(exchangeId: string, params?: PaginationParams): Promise<any>;
  getStockBySymbol(symbol: string): Promise<any>;
  saveStockData(): Promise<any>;
}

export type StockModelType = Model<IStockModel>;
export type ExchangeModelType = Model<IExchangeModel>;
