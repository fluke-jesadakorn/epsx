import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import { Document, Schema as MongooseSchema } from "mongoose";

@Schema({ timestamps: true })
export class Stock extends Document {
  @Prop({ required: true, unique: true })
  symbol!: string;

  @Prop({ required: true })
  company_name!: string;

  @Prop({ type: MongooseSchema.Types.ObjectId, ref: "Exchange" })
  exchange!: MongooseSchema.Types.ObjectId;
}

export const StockSchema = SchemaFactory.createForClass(Stock);
