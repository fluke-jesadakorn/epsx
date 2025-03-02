import { Injectable, NotFoundException, BadRequestException } from '@nestjs/common';
import { auth } from '../../shared/firebase-admin';
import { UserRole } from '../../shared/guards/role.guard';

@Injectable()
export class AuthService {
  async assignUserRole(userId: string, role: UserRole): Promise<void> {
    try {
      const user = await auth().getUser(userId);
      
      let claims = {
        admin: false,
        premium: false,
        basic: false
      };

      // Set claims based on role
      switch (role) {
        case UserRole.PREMIUM:
          claims.premium = true;
          break;
        case UserRole.BASIC:
          claims.basic = true;
          break;
        // Public role has no special claims
        default:
          break;
      }

      await auth().setCustomUserClaims(userId, claims);
    } catch (error: any) {
      if (error?.code === 'auth/user-not-found') {
        throw new NotFoundException('User not found');
      }
      throw new BadRequestException('Failed to assign role');
    }
  }

  async getUserRole(userId: string): Promise<{ userId: string; email: string | undefined; role: UserRole }> {
    try {
      const user = await auth().getUser(userId);
      const claims = user.customClaims || {};

      let role = UserRole.PUBLIC;
      if (claims.premium) {
        role = UserRole.PREMIUM;
      } else if (claims.basic) {
        role = UserRole.BASIC;
      }

      return {
        userId: user.uid,
        email: user.email,
        role
      };
    } catch (error: any) {
      if (error?.code === 'auth/user-not-found') {
        throw new NotFoundException('User not found');
      }
      throw new BadRequestException('Failed to get user role');
    }
  }

  async listUsers(maxResults: number = 1000): Promise<Array<{ userId: string; email: string | undefined; role: UserRole }>> {
    try {
      const listUsersResult = await auth().listUsers(maxResults);
      
      return listUsersResult.users.map(user => {
        const claims = user.customClaims || {};
        let role = UserRole.PUBLIC;
        
        if (claims.premium) {
          role = UserRole.PREMIUM;
        } else if (claims.basic) {
          role = UserRole.BASIC;
        }

        return {
          userId: user.uid,
          email: user.email,
          role
        };
      });
    } catch (error) {
      throw new BadRequestException('Failed to list users');
    }
  }
}
