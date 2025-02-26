import { Prop, Schema, SchemaFactory } from "@nestjs/mongoose";
import { Document, Schema as MongooseSchema } from "mongoose";
import { Stock } from "../stock/schema";
import { ApiProperty } from "@nestjs/swagger";

@Schema({ timestamps: true })
export class Financial extends Document {
  @ApiProperty({
    description: 'Reference to the associated stock',
    type: 'string',
    format: 'mongoose-id',
    example: '507f1f77bcf86cd799439011'
  })
  @Prop({ type: MongooseSchema.Types.ObjectId, ref: "Stock", required: true })
  stock!: Stock;

  @ApiProperty({
    description: 'Fiscal year of the financial data',
    type: 'number',
    example: 2024,
    minimum: 1900
  })
  @Prop({ required: true })
  fiscal_year!: number;

  @ApiProperty({
    description: 'Fiscal quarter (1-4)',
    type: 'number',
    example: 4,
    minimum: 1,
    maximum: 4
  })
  @Prop({ required: true })
  fiscal_quarter!: number;

  @ApiProperty({
    description: 'Diluted Earnings Per Share value',
    type: 'number',
    example: 2.45,
    format: 'float'
  })
  @Prop({ required: true })
  eps_diluted!: number;

  @ApiProperty({
    description: 'Date when the financial report was published',
    type: 'string',
    format: 'date-time',
    example: '2024-02-26T08:30:00.000Z'
  })
  @Prop({ required: true })
  report_date!: Date;
}

export const FinancialSchema = SchemaFactory.createForClass(Financial);
