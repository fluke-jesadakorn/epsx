import { Document, Types } from 'mongoose';
import { IStockDocument } from '@epsx/shared';

export interface StockDocument extends Document, IStockDocument {
  _id: Types.ObjectId;
  symbol: string;
  company_name: string;
  market_cap?: number;
  sector?: string;
  volume?: number;
  exchange: Types.ObjectId;
  createdAt: Date;
  updatedAt: Date;
}
