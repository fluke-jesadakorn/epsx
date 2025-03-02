import { Module } from '@nestjs/common';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { AdminGuard } from './guards/admin.guard';

@Module({
  controllers: [AuthController],
  providers: [
    AuthService,
    AdminGuard
  ],
  exports: [AuthService],
})
export class AuthModule {}
