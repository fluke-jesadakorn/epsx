/**
 * Shared User Form Utilities
 * Common functions and helpers extracted from UserForms.tsx
 */

import React from 'react';
import { toast } from '@/hooks/use-toast';
import { logger } from '@/lib/logger';
import type { User } from '@/types/core';

// Common form state management
export interface FormStepState {
  formStep: number;
  previewMode: boolean;
  isSubmitting: boolean;
}

export const createFormStepActions = (
  setFormStep: React.Dispatch<React.SetStateAction<number>>,
  setPreviewMode: React.Dispatch<React.SetStateAction<boolean>>
) => ({
  handleNextStep: () => {
    setFormStep((prev) => (prev < 3 ? prev + 1 : prev));
  },
  handlePrevStep: () => {
    setFormStep((prev) => (prev > 1 ? prev - 1 : prev));
  },
  handlePreviewUser: () => {
    setPreviewMode(true);
  }
});

// Common toast messages
export const showSuccessToast = (message: string, description?: string) => {
  toast({
    title: message,
    description,
    variant: "default",
  });
};

export const showErrorToast = (message: string, description?: string) => {
  toast({
    title: message,
    description,
    variant: "destructive",
  });
};

// User data transformation helpers
export const createUserFromFormData = (data: any): User => ({
  id: crypto.randomUUID(),
  email: data.email,
  name: data.displayName || `${data.firstName || ''} ${data.lastName || ''}`.trim(),
  firstName: data.firstName,
  lastName: data.lastName,
  displayName: data.displayName,
  role: data.role,
  packageTier: data.packageTier,
  permissions: data.permissions,
  isActive: data.isActive,
  wallet_address: data.wallet_address || null,
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString()
});

export const updateUserFromFormData = (existingUser: User, data: any): User => ({
  ...existingUser,
  email: data.email || existingUser.email,
  name: data.displayName || `${data.firstName || ''} ${data.lastName || ''}`.trim(),
  firstName: data.firstName,
  lastName: data.lastName,
  displayName: data.displayName,
  role: data.role || existingUser.role,
  packageTier: data.packageTier || existingUser.packageTier,
  permissions: data.permissions || existingUser.permissions,
  isActive: data.isActive !== undefined ? data.isActive : existingUser.isActive,
  updatedAt: new Date().toISOString()
});

// Error handling utilities
export const handleApiError = (error: any, context: string) => {
  logger.error(`${context} error`, { error });
  showErrorToast(
    "Error",
    "Network error. Please check your connection and try again."
  );
};

// Form validation helpers
export const validateEmailUniqueness = (email: string, existingUsers: User[], excludeUserId?: string): boolean => {
  const existingUser = existingUsers.find(user => 
    user.email?.toLowerCase() === email.toLowerCase() && user.id !== excludeUserId
  );
  
  if (existingUser) {
    showErrorToast("Email already exists", "This email address is already registered to another user.");
    return false;
  }
  
  return true;
};

// Permission helpers
export const formatPermissionsForDisplay = (permissions: string[]): string => {
  if (permissions.length === 0) return 'No permissions';
  if (permissions.length <= 3) return permissions.join(', ');
  return `${permissions.slice(0, 3).join(', ')} +${permissions.length - 3} more`;
};