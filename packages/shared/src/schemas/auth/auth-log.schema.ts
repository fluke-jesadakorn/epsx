import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document } from 'mongoose';

export interface GeoLocation {
  country?: string;
  city?: string;
  latitude?: number;
  longitude?: number;
}

@Schema({ timestamps: true })
export class AuthLog {
  @Prop({ required: true })
  action!: string;

  @Prop({ required: true })
  status!: string;

  @Prop()
  userId?: string;

  @Prop()
  errorCode?: string;

  @Prop()
  errorDetails?: string;

  @Prop({ type: Object })
  metadata?: Record<string, any>;

  @Prop()
  ipAddress?: string;

  @Prop()
  userAgent?: string;

  @Prop()
  sessionId?: string;

  @Prop({ type: Object })
  geoLocation?: GeoLocation;

  @Prop({ default: false })
  isSuspicious!: boolean;

  @Prop({ type: Date, default: Date.now })
  timestamp!: Date;
}

export type AuthLogDocument = AuthLog & Document;
export const AuthLogSchema = SchemaFactory.createForClass(AuthLog);
