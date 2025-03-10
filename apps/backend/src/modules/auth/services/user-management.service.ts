import { Injectable, UnauthorizedException, Logger, InternalServerErrorException } from '@nestjs/common';
import { FirebaseAdminService } from '../../../shared/firebase-admin';
import { AuthLoggerService } from './auth-logger.service';
import { RoleService } from './role.service';
import { UserRole } from '../../../shared/types/roles.enum';
import {
  UsersResponseDto,
  RoleAssignmentResponse,
  RoleAssignmentMetadata
} from '../interfaces/user-management.interface';

@Injectable()
export class UserManagementService {
  private readonly logger = new Logger(UserManagementService.name);

  constructor(
    private readonly firebaseAdmin: FirebaseAdminService,
    private readonly authLogger: AuthLoggerService,
    private readonly roleService: RoleService
  ) {}

  async assignUserRole(userId: string, role: UserRole, adminId: string): Promise<RoleAssignmentResponse> {
    try {
      await this.roleService.assignRole(userId, role);

      const metadata: RoleAssignmentMetadata = {
        roleUpdate: true,
        newRole: role,
        adminId
      };

      await this.authLogger.logAuthEvent({
        userId,
        action: 'refresh',
        status: 'success',
        metadata
      });

      return { success: true };
    } catch (error) {
      this.logger.error(`Failed to assign role to user ${userId}:`, error);
      if (error instanceof UnauthorizedException) {
        throw error;
      }
      throw new InternalServerErrorException("Failed to assign role");
    }
  }

  async getUsers(): Promise<UsersResponseDto> {
    try {
      return await this.roleService.getAllUsersWithRoles();
    } catch (error) {
      this.logger.error('Failed to fetch users:', error);
      if (error instanceof UnauthorizedException) {
        throw error;
      }
      throw new InternalServerErrorException("Failed to fetch users");
    }
  }
}
