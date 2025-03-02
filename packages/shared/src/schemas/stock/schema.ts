import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import type { HydratedDocument } from "mongoose";
import { Schema as MongooseSchema } from "mongoose";

export interface IStock {
  symbol: string;
  company_name: string;
  exchange: MongooseSchema.Types.ObjectId;
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
}

export const StockSchema = SchemaFactory.createForClass(Stock);
