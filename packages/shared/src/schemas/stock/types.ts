import { Document, Schema } from 'mongoose';

export interface IStock {
  symbol: string;
  company_name: string;
  exchange: Schema.Types.ObjectId;
}

export type StockDocument = IStock & Document;
