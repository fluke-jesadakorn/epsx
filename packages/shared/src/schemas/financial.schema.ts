import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document, Schema as MongooseSchema } from 'mongoose';
import { Stock } from './stock.schema';

@Schema({ timestamps: true })
export class Financial extends Document {
  @Prop({ type: MongooseSchema.Types.ObjectId, ref: 'Stock', required: true })
  stock!: Stock;

  @Prop({ required: true })
  fiscal_year!: number;

  @Prop({ required: true })
  fiscal_quarter!: number;

  @Prop({ required: true })
  eps_diluted!: number;

  @Prop({ required: true })
  report_date!: Date;
}

export const FinancialSchema = SchemaFactory.createForClass(Financial);
