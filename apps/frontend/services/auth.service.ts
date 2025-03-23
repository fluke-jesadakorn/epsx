import type { LogoutResponse } from "@/types/auth";
import { UserRole } from "@/types/auth/roles";
import { TokenFeature } from "@/types/auth/features";
import type { Permission } from "@/types/auth/features";

interface AuthStatus {
  isLoggedIn: boolean;
  userEmail: string | null;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  isAdmin: boolean;
}

class AuthService {
  private getDefaultStatus(): AuthStatus {
    return {
      isLoggedIn: false,
      userEmail: null,
      role: UserRole.GUEST,
      tokenBalance: 0,
      features: [],
      permissions: [],
      isAdmin: false
    };
  }

  async checkAuthStatus(): Promise<AuthStatus> {
    try {
      const { verifyAuth } = await import('@/app/actions/auth-server');
      const { isLoggedIn, userEmail, role, tokenBalance, features, permissions } = await verifyAuth();
      
      return {
        isLoggedIn,
        userEmail,
        role: role || UserRole.GUEST,
        tokenBalance: tokenBalance || 0,
        features: features || [],
        permissions: permissions || [],
        isAdmin: role === UserRole.ADMINISTRATOR
      };
    } catch (error) {
      console.error('Error checking auth status:', error);
      return this.getDefaultStatus();
    }
  }

  async logout(): Promise<LogoutResponse> {
    try {
      const { signOut } = await import('@/app/actions/auth-server');
      await signOut();
      return {
        success: true,
        redirectUrl: "/login"
      };
    } catch (error) {
      console.error("Error during logout:", error);
      return {
        success: false,
        redirectUrl: "/login"
      };
    }
  }
}

export const authService = new AuthService();
