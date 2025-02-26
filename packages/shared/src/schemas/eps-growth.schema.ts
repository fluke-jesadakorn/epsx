import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';

@Schema({ timestamps: true })
export class EpsGrowth extends Document {
  @Prop({ required: true })
  symbol!: string;

  @Prop({ required: true })
  company_name!: string;

  @Prop({ required: true })
  market_code!: string;

  @Prop({ required: true })
  eps_diluted!: number;

  @Prop({ required: true })
  previous_eps_diluted!: number;

  @Prop({ required: true })
  eps_growth!: number;

  @Prop({ required: true, type: Date })
  report_date!: Date;

  @Prop({ required: true })
  year!: number;

  @Prop({ required: true })
  quarter!: number;
}

export const EpsGrowthSchema = SchemaFactory.createForClass(EpsGrowth);
