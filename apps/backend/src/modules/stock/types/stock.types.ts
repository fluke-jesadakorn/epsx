import { Stock } from '@epsx/shared';
import { Document, Schema as MongooseSchema } from 'mongoose';

export interface StockDocument extends Stock, Document {
  _id: MongooseSchema.Types.ObjectId;
  market_cap?: number;
  sector?: string;
  volume?: number;
  createdAt: Date;
  updatedAt: Date;
  __v: number;
}
