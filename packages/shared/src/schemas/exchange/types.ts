import { Document, Schema } from 'mongoose';

export interface IExchange {
  market_code: string;
  name: string;
  country: string;
  description?: string;
  timezone?: string;
  currency?: string;
  stocks?: Schema.Types.ObjectId[];
}

export type ExchangeDocument = IExchange & Document;
