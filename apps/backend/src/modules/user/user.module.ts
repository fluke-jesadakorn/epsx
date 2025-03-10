import { Module } from '@nestjs/common';
import { UserController } from './controllers/user.controller';
import { UserService } from './services/user.service';
import { FirebaseAdminService } from '../../shared/firebase-admin';

@Module({
  controllers: [UserController],
  providers: [UserService, FirebaseAdminService],
  exports: [UserService]
})
export class UserModule {}
