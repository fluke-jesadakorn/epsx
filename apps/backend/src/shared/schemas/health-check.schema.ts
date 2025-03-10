import { Prop, Schema, SchemaFactory } from '@nestjs/mongoose';
import { Document, Schema as MongooseSchema } from 'mongoose';

@Schema({ timestamps: true })
export class HealthCheckLog extends Document {
  @Prop({ required: true })
  status!: 'healthy' | 'degraded' | 'unhealthy';

  @Prop({ required: true })
  timestamp!: Date;

  @Prop()
  responseTime?: number;

  @Prop({ required: true })
  isConnected!: boolean;

  @Prop({
    type: {
      connectionsActive: Number,
      connectionsAvailable: Number,
      connectionsCreated: Number,
      connectionsReused: Number,
      operationsQueued: Number,
    },
  })
  metrics?: {
    connectionsActive: number;
    connectionsAvailable: number;
    connectionsCreated: number;
    connectionsReused: number;
    operationsQueued: number;
  };
}

@Schema({ timestamps: true })
export class ConnectionEvent extends Document {
  @Prop({ required: true })
  type!: 'connected' | 'disconnected' | 'reconnected' | 'error';

  @Prop({ required: true })
  timestamp!: Date;

  @Prop({ required: true })
  message!: string;

  @Prop({
    type: {
      name: String,
      message: String,
      stack: String,
    },
  })
  error?: {
    name: string;
    message: string;
    stack?: string;
  };
}

export const HealthCheckLogSchema = SchemaFactory.createForClass(HealthCheckLog);
export const ConnectionEventSchema = SchemaFactory.createForClass(ConnectionEvent);

// Add indexes
HealthCheckLogSchema.index({ timestamp: -1 });
HealthCheckLogSchema.index({ status: 1, timestamp: -1 });

ConnectionEventSchema.index({ timestamp: -1 });
ConnectionEventSchema.index({ type: 1, timestamp: -1 });

// Add methods
HealthCheckLogSchema.methods.toPublicResponse = function() {
  const { _id, __v, ...publicData } = this.toObject();
  return publicData;
};

ConnectionEventSchema.methods.toPublicResponse = function() {
  const { _id, __v, ...publicData } = this.toObject();
  return publicData;
};

// Add statics
HealthCheckLogSchema.statics.getLatestStatus = async function() {
  return this.findOne().sort({ timestamp: -1 }).exec();
};

HealthCheckLogSchema.statics.getStatusHistory = async function(limit = 100) {
  return this.find()
    .sort({ timestamp: -1 })
    .limit(limit)
    .exec();
};

ConnectionEventSchema.statics.getRecentEvents = async function(limit = 100) {
  return this.find()
    .sort({ timestamp: -1 })
    .limit(limit)
    .exec();
};

// Types
export type HealthCheckLogDocument = HealthCheckLog & Document;
export type ConnectionEventDocument = ConnectionEvent & Document;
