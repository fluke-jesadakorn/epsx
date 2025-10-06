/**
 * USER DATA HOOKS
 *
 * React hooks for user profile, settings, and subscription management.
 * Provides automatic data fetching, caching, and mutation handling.
 */

'use client';

import { useState, useEffect, useCallback } from 'react';
import { useApiClient } from './useApiClient';
import type {
  UserProfile,
  UserSettings,
  UpdateProfileRequest,
  UpdateSettingsRequest,
  SubscriptionInfo,
  UserApiKey
} from '../api/users';
import type { ApiResponse } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

interface UseDataResult<T> {
  data: T | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

interface UseMutationResult<TData, TVariables> {
  mutate: (variables: TVariables) => Promise<TData>;
  data: TData | null;
  loading: boolean;
  error: Error | null;
  reset: () => void;
}

// ============================================================================
// PROFILE HOOKS
// ============================================================================

/**
 * Get current user profile
 *
 * @example
 * const { data: profile, loading, error } = useUserProfile();
 */
export function useUserProfile(): UseDataResult<UserProfile> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchProfile = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.getProfile();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch profile');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [users]);

  useEffect(() => {
    fetchProfile();
  }, [fetchProfile]);

  return { data, loading, error, refetch: fetchProfile };
}

/**
 * Update user profile
 *
 * @example
 * const { mutate: updateProfile, loading } = useUpdateProfile();
 * await updateProfile({ display_name: 'New Name' });
 */
export function useUpdateProfile(): UseMutationResult<UserProfile, UpdateProfileRequest> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (variables: UpdateProfileRequest) => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.updateProfile(variables);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to update profile');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [users]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// SETTINGS HOOKS
// ============================================================================

/**
 * Get user settings
 *
 * @example
 * const { data: settings, refetch } = useUserSettings();
 */
export function useUserSettings(): UseDataResult<UserSettings> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchSettings = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.getSettings();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch settings');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [users]);

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  return { data, loading, error, refetch: fetchSettings };
}

/**
 * Update user settings
 *
 * @example
 * const { mutate: updateSettings } = useUpdateSettings();
 * await updateSettings({ notifications_enabled: true });
 */
export function useUpdateSettings(): UseMutationResult<UserSettings, UpdateSettingsRequest> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserSettings | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (variables: UpdateSettingsRequest) => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.updateSettings(variables);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to update settings');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [users]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// SUBSCRIPTION HOOKS
// ============================================================================

/**
 * Get user subscriptions
 *
 * @example
 * const { data: subscriptions, loading } = useSubscriptions();
 */
export function useSubscriptions(): UseDataResult<SubscriptionInfo[]> {
  const { users } = useApiClient();
  const [data, setData] = useState<SubscriptionInfo[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchSubscriptions = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.getSubscriptions();
      if (response.success && response.data) {
        setData(response.data);
      } else {
        throw new Error(response.error || 'Failed to fetch subscriptions');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [users]);

  useEffect(() => {
    fetchSubscriptions();
  }, [fetchSubscriptions]);

  return { data, loading, error, refetch: fetchSubscriptions };
}

/**
 * Subscribe to plan
 *
 * @example
 * const { mutate: subscribe, loading } = useSubscribeToPlan();
 * await subscribe('premium-plan-id');
 */
export function useSubscribeToPlan(): UseMutationResult<SubscriptionInfo, string> {
  const { users } = useApiClient();
  const [data, setData] = useState<SubscriptionInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (planId: string) => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.subscribeToPlan(planId);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to subscribe');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [users]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// API KEY HOOKS
// ============================================================================

/**
 * Get user API keys
 *
 * @example
 * const { data: apiKeys, refetch } = useApiKeys();
 */
export function useApiKeys(): UseDataResult<UserApiKey[]> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserApiKey[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchApiKeys = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.getApiKeys();
      if (response.success && response.data) {
        setData(response.data.api_keys);
      } else {
        throw new Error(response.error || 'Failed to fetch API keys');
      }
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [users]);

  useEffect(() => {
    fetchApiKeys();
  }, [fetchApiKeys]);

  return { data, loading, error, refetch: fetchApiKeys };
}

/**
 * Create API key
 *
 * @example
 * const { mutate: createKey } = useCreateApiKey();
 * await createKey({ name: 'My API Key', scopes: ['read'] });
 */
export function useCreateApiKey(): UseMutationResult<UserApiKey, { name: string; scopes?: string[] }> {
  const { users } = useApiClient();
  const [data, setData] = useState<UserApiKey | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async ({ name, scopes }: { name: string; scopes?: string[] }) => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.createApiKey(name, scopes);
      if (response.success && response.data) {
        setData(response.data);
        return response.data;
      }
      throw new Error(response.error || 'Failed to create API key');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [users]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

/**
 * Delete API key
 *
 * @example
 * const { mutate: deleteKey } = useDeleteApiKey();
 * await deleteKey('key-id');
 */
export function useDeleteApiKey(): UseMutationResult<boolean, string> {
  const { users } = useApiClient();
  const [data, setData] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const mutate = useCallback(async (keyId: string) => {
    try {
      setLoading(true);
      setError(null);
      const response = await users.deleteApiKey(keyId);
      if (response.success && response.data) {
        setData(response.data.deleted);
        return response.data.deleted;
      }
      throw new Error(response.error || 'Failed to delete API key');
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Unknown error');
      setError(error);
      throw error;
    } finally {
      setLoading(false);
    }
  }, [users]);

  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setLoading(false);
  }, []);

  return { mutate, data, loading, error, reset };
}

// ============================================================================
// EXPORTS
// ============================================================================

export default useUserProfile;
