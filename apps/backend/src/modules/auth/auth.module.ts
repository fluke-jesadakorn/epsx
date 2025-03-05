import { Module } from '@nestjs/common';
import { AuthService } from './auth.service';
import { AuthController } from './auth.controller';
import { FirebaseAdminService } from '../../shared/firebase-admin';

@Module({
  controllers: [AuthController],
  providers: [AuthService, FirebaseAdminService],
  exports: [AuthService],
})
export class AuthModule {}
