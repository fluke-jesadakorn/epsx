import { Schema } from "mongoose";
import { IStock } from "./types";

const schema = new Schema<IStock>(
  {
    symbol: { type: String, required: true, unique: true },
    company_name: { type: String, required: true },
    exchange: { type: Schema.Types.ObjectId, ref: "Exchange", required: true },
  },
  {
    timestamps: true,
    collection: "stocks",
  }
);

// Add indexes for better query performance
schema.index({ symbol: 1 }, { unique: true });
schema.index({ exchange: 1 });

export const StockSchema = schema;
