import { Schema, Prop, SchemaFactory } from '@nestjs/mongoose';
import { HydratedDocument } from 'mongoose';

export interface IExchange {
  exchange_name: string;
  country: string;
  market_code: string;
  currency: string;
  exchange_url?: string;
  timezone?: string;
}

export type ExchangeDocument = HydratedDocument<Exchange>;

@Schema({ timestamps: true })
export class Exchange {
  @Prop({ type: String, required: true })
  exchange_name: string;

  @Prop({ type: String, required: true })
  country: string;

  @Prop({ type: String, required: true, unique: true })
  market_code: string;

  @Prop({ type: String, required: true })
  currency: string;

  @Prop({ type: String })
  exchange_url?: string;

  @Prop({ type: String, default: "UTC" })
  timezone: string;
}

export const ExchangeSchema = SchemaFactory.createForClass(Exchange);
