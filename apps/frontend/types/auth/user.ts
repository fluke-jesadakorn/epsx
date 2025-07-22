import type { USDTDetails } from "@/types/userLevel";
import type { UserSubscription } from "@epsx/types";

export interface User {
  id: string;
  email: string;
  createdAt: string;
  updatedAt: string;
  emailVerified?: boolean;
  role: 'USER' | 'ADMIN';
  displayName?: string;
  photoURL?: string;
  usdtDetails?: USDTDetails;
  subscription?: UserSubscription; // New payment system
  token_balance?: number; // For legacy token-based features
  features?: string[]; // For legacy feature system
  permissions?: string[]; // For legacy permission system
}

export interface UserCredentials {
  email: string;
  password: string;
}
