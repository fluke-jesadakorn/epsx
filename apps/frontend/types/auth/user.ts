import type { USDTDetails } from "@/types/userLevel";
import type { UserSubscription } from "@epsx/types";

export interface User {
  id: string;
  email: string;
  createdAt: string;
  updatedAt: string;
  emailVerified?: boolean;
  role: 'USER' | 'ADMIN';
  displayName?: string | undefined;
  photoURL?: string | undefined;
  usdtDetails?: USDTDetails | undefined;
  subscription?: UserSubscription | undefined; // New payment system
  token_balance?: number | undefined; // For legacy token-based features
  features?: string[] | undefined; // For legacy feature system
  permissions?: string[] | undefined; // For legacy permission system
}

export interface UserCredentials {
  email: string;
  password: string;
}
