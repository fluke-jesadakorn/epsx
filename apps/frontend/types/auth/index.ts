import { UserRole } from "./roles";

export interface AuthUser {
  userEmail: string | null;
  isLoggedIn: boolean;
  isAdmin: boolean;
  role?: UserRole;
}

export interface AuthStatus {
  isAuthenticated: boolean;
  email: string | null;
  role?: UserRole;
}

export interface AuthContextType extends AuthUser {}

export interface AuthProviderProps {
  children: React.ReactNode;
}

export interface LoginResponse {
  success: boolean;
  redirectUrl: string;
}

export interface LogoutResponse extends LoginResponse {}
