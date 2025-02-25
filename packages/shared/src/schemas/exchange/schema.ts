import { Schema } from "mongoose";
import { IExchange } from "./types";

const schema = new Schema<IExchange>(
  {
    market_code: { type: String, required: true, unique: true },
    name: { type: String, required: true },
    country: { type: String, required: true },
    description: { type: String, default: "" },
    timezone: { type: String, default: "" },
    currency: { type: String, default: "" },
    stocks: [{ type: Schema.Types.ObjectId, ref: "Stock" }],
  },
  {
    timestamps: true,
    collection: "exchanges",
  }
);

// Add indexes for better query performance
schema.index({ market_code: 1 }, { unique: true });
schema.index({ country: 1 });

export const ExchangeSchema = schema;
