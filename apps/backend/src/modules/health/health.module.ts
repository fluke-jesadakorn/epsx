import { Module } from '@nestjs/common';
import { HealthController } from '../../shared/controllers/health.controller';
import { DatabaseConfigService } from '../../shared/services/database-config.service';
import { ErrorHandlingService } from '../../shared/services/error-handling.service';
import { MongooseModule } from '@nestjs/mongoose';
import { 
  HealthCheckLog, 
  HealthCheckLogSchema,
  ConnectionEvent,
  ConnectionEventSchema 
} from '../../shared/schemas/health-check.schema';

@Module({
  imports: [
    MongooseModule.forFeature([
      { name: HealthCheckLog.name, schema: HealthCheckLogSchema },
      { name: ConnectionEvent.name, schema: ConnectionEventSchema }
    ])
  ],
  controllers: [HealthController],
  providers: [
    DatabaseConfigService,
    ErrorHandlingService
  ],
  exports: [
    DatabaseConfigService,
    ErrorHandlingService
  ]
})
export class HealthModule {}
