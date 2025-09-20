// Permission-based Payment Types
import { PermissionTemplateName } from '@/app/constants/packages';

export interface PaymentPlan {
  id: string;
  name: string;
  permissionTemplate: PermissionTemplateName;
  permissions: string[];
  displayTier: string;
  price: number;
  currency: string;
  features: string[];
  duration: number;
  color: string;
}

export interface User {
  id: string;
  email: string;
  name?: string;
  permissions?: string[];
  permissionTemplate?: PermissionTemplateName;
  displayTier?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface PaymentStatus {
  id: string;
  status: 'pending' | 'completed' | 'failed' | 'cancelled';
  amount: number;
  currency: string;
  createdAt: string;
  userId?: string;
  permissionTemplate?: PermissionTemplateName;
}

export interface PaymentTransaction {
  id: string;
  amount: number;
  currency: string;
  status: string;
  createdAt: string;
  description?: string;
  userId?: string;
  permissionTemplate?: PermissionTemplateName;
}