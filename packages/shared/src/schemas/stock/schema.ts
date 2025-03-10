import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import type { HydratedDocument } from "mongoose";
import { Schema as MongooseSchema } from "mongoose";

export interface IStock {
  symbol: string;
  company_name: string;
  exchange: MongooseSchema.Types.ObjectId;
  market_cap?: number;
  sector?: string;
  industry?: string;
  website?: string;
  description?: string;
  ceo?: string;
  employees?: number;
  headquarters?: string;
}

export type StockDocument = HydratedDocument<Stock>;

@Schema({ timestamps: true })
export class Stock implements IStock {
  @Prop({ type: String, required: true, unique: true })
  symbol!: string;

  @Prop({ type: String, required: true })
  company_name!: string;

  @Prop({ type: MongooseSchema.Types.ObjectId, ref: "Exchange", required: true })
  exchange!: MongooseSchema.Types.ObjectId;

  @Prop({ type: Number })
  market_cap?: number;

  @Prop({ type: String })
  sector?: string;

  @Prop({ type: String })
  industry?: string;

  @Prop({ type: String })
  website?: string;

  @Prop({ type: String })
  description?: string;

  @Prop({ type: String })
  ceo?: string;

  @Prop({ type: Number })
  employees?: number;

  @Prop({ type: String })
  headquarters?: string;
}

export const StockSchema = SchemaFactory.createForClass(Stock);
