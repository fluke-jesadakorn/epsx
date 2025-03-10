import { Injectable, Logger } from '@nestjs/common';
import { FirebaseAdminService } from '../../../shared/firebase-admin';
import { UserRole } from '../../../shared/types/roles.enum';
import { UsersResponseDto, UserWithRoleDto } from '../interfaces/user-management.interface';

@Injectable()
export class RoleService {
  private readonly logger = new Logger(RoleService.name);

  constructor(
    private readonly firebaseAdmin: FirebaseAdminService
  ) {}

  determineUserRole(customClaims: Record<string, any>): UserRole {
    if (customClaims.admin) {
      return UserRole.ADMINISTRATOR;
    }
    if (customClaims.token_holder) {
      return UserRole.TOKEN_HOLDER;
    }
    if (customClaims.premium) {
      return UserRole.PREMIUM_USER;
    }
    if (customClaims.basic) {
      return UserRole.REGISTERED_USER;
    }
    return UserRole.GUEST;
  }

  async assignRole(userId: string, role: UserRole): Promise<void> {
    const claims: Record<string, boolean | string> = {
      admin: false,
      token_holder: false,
      premium: false,
      basic: false,
      role: role
    };

    switch (role) {
      case UserRole.ADMINISTRATOR:
        claims.admin = true;
        break;
      case UserRole.TOKEN_HOLDER:
        claims.token_holder = true;
        break;
      case UserRole.PREMIUM_USER:
        claims.premium = true;
        break;
      case UserRole.REGISTERED_USER:
        claims.basic = true;
        break;
    }

    await this.firebaseAdmin.setCustomUserClaims(userId, claims);
  }

  async getAllUsersWithRoles(): Promise<UsersResponseDto> {
    const { users } = await this.firebaseAdmin.auth.listUsers();
    
    const usersWithRoles: UserWithRoleDto[] = users.map((user) => {
      const customClaims = user.customClaims || {};
      const userRole = this.determineUserRole(customClaims);

      return {
        userId: user.uid,
        email: user.email || '',
        role: userRole,
        features: customClaims.features || [],
        permissions: customClaims.permissions || [],
        tokenBalance: customClaims.tokenBalance || 0
      };
    });

    return {
      users: usersWithRoles
    };
  }
}
