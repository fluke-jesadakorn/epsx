import { Module, Global, OnModuleInit } from '@nestjs/common';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { MongooseModule, getModelToken, InjectModel } from '@nestjs/mongoose';
import { Model } from 'mongoose';
import { ErrorHandlingService } from './services/error-handling.service';
import { DatabaseConfigService } from './services/database-config.service';
import { FirebaseAdminService } from './firebase-admin';
import { AuthService } from './auth';
import { HealthController } from './controllers/health.controller';
import {
  HealthCheckLog,
  HealthCheckLogSchema,
  HealthCheckLogDocument,
  ConnectionEvent,
  ConnectionEventSchema,
  ConnectionEventDocument
} from './schemas/health-check.schema';

@Global()
@Module({
  imports: [
    ConfigModule,
    MongooseModule.forFeature([
      { name: HealthCheckLog.name, schema: HealthCheckLogSchema },
      { name: ConnectionEvent.name, schema: ConnectionEventSchema }
    ])
  ],
  controllers: [
    HealthController
  ],
  providers: [
    ErrorHandlingService,
    DatabaseConfigService,
    FirebaseAdminService,
    AuthService,
    ConfigService
  ],
  exports: [
    ErrorHandlingService,
    DatabaseConfigService,
    FirebaseAdminService,
    AuthService,
    MongooseModule
  ]
})
export class SharedModule implements OnModuleInit {
  constructor(
    private databaseConfigService: DatabaseConfigService,
    @InjectModel(HealthCheckLog.name) private healthCheckModel: Model<HealthCheckLogDocument>,
    @InjectModel(ConnectionEvent.name) private connectionEventModel: Model<ConnectionEventDocument>
  ) {}

  onModuleInit() {
    this.databaseConfigService.setModels(this.healthCheckModel, this.connectionEventModel);
  }
}
