import { Schema, Prop, SchemaFactory } from "@nestjs/mongoose";
import { HydratedDocument, Schema as MongooseSchema } from "mongoose";

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
export class Exchange implements IExchange {
  @Prop({ required: true, type: String })
  exchange_name!: string;

  @Prop({ required: true, type: String })
  country!: string;

  @Prop({ required: true, type: String })
  market_code!: string;

  @Prop({ required: true, type: String })
  currency!: string;

  @Prop({ type: String })
  exchange_url?: string;

  @Prop({ type: String, default: "UTC" })
  timezone: string = "UTC";

  constructor(partial: Partial<IExchange> = {}) {
    Object.assign(this, partial);
  }
}

export const ExchangeSchema = SchemaFactory.createForClass(Exchange);

ExchangeSchema.virtual("id").get(function () {
  return this._id.toHexString();
});

ExchangeSchema.set("toJSON", {
  virtuals: true,
  transform: (_, ret) => {
    delete ret._id;
    delete ret.__v;
    return ret;
  },
});

// Ensure indexes
ExchangeSchema.index({ market_code: 1 }, { unique: true });
