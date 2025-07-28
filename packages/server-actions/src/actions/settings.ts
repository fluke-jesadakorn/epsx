'use server';

import { serverGet, serverPost, serverPut } from '../core/request';

// System Settings Actions
export async function getSystemConfig() {
  try {
    return await serverGet('/api/v1/settings/system');
  } catch (error) {
    console.error('Error fetching system config:', error);
    return {};
  }
}

export async function updateSettings(data: {
  category: string;
  settings: Record<string, any>;
  updatedBy: string;
}) {
  try {
    return await serverPut('/api/v1/settings/update', data);
  } catch (error) {
    console.error('Error updating settings:', error);
    throw error;
  }
}

export async function getSettingsByCategory(category: string) {
  try {
    return await serverGet(`/api/v1/settings/category/${category}`);
  } catch (error) {
    console.error('Error fetching settings by category:', error);
    return {};
  }
}

// User-specific Settings
export async function getUserSettings(userId: string) {
  try {
    return await serverGet(`/api/v1/settings/users/${userId}`);
  } catch (error) {
    console.error('Error fetching user settings:', error);
    return {};
  }
}

export async function updateUserSettings(data: {
  userId: string;
  settings: Record<string, any>;
}) {
  try {
    return await serverPut(`/api/v1/settings/users/${data.userId}`, data);
  } catch (error) {
    console.error('Error updating user settings:', error);
    throw error;
  }
}

// Feature Flags
export async function getFeatureFlags(userId?: string) {
  try {
    const endpoint = userId 
      ? `/api/v1/settings/feature-flags/${userId}`
      : '/api/v1/settings/feature-flags';
    return await serverGet(endpoint);
  } catch (error) {
    console.error('Error fetching feature flags:', error);
    return {};
  }
}

export async function updateFeatureFlag(data: {
  flagName: string;
  enabled: boolean;
  userId?: string;
  updatedBy: string;
}) {
  try {
    return await serverPut('/api/v1/settings/feature-flags/update', data);
  } catch (error) {
    console.error('Error updating feature flag:', error);
    throw error;
  }
}

// Configuration Templates
export async function getConfigTemplates() {
  try {
    return await serverGet('/api/v1/settings/templates');
  } catch (error) {
    console.error('Error fetching config templates:', error);
    return [];
  }
}

export async function applyConfigTemplate(data: {
  templateId: string;
  userId?: string;
  appliedBy: string;
  overrides?: Record<string, any>;
}) {
  try {
    return await serverPost('/api/v1/settings/templates/apply', data);
  } catch (error) {
    console.error('Error applying config template:', error);
    throw error;
  }
}

// Environment Configuration
export async function getEnvironmentConfig() {
  try {
    return await serverGet('/api/v1/settings/environment');
  } catch (error) {
    console.error('Error fetching environment config:', error);
    return {};
  }
}

export async function updateEnvironmentConfig(data: {
  config: Record<string, any>;
  updatedBy: string;
  requiresRestart?: boolean;
}) {
  try {
    return await serverPut('/api/v1/settings/environment/update', data);
  } catch (error) {
    console.error('Error updating environment config:', error);
    throw error;
  }
}