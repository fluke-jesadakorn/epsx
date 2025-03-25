import { UserRole } from './roles';
import type { TokenFeature, Permission } from './features';

export interface RedirectResponse {
  redirect: string;
}

export interface AuthResponse {
  token: string;
  email: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  redirectUrl?: string;
}

export type AuthActionResponse = RedirectResponse | null;
