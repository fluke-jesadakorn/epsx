import { useState, useEffect, useCallback } from 'react';
import { storage } from '@/lib/state';

interface PersistentStateOptions<T> {
  key: string;
  defaultValue: T;
  storage?: 'localStorage' | 'sessionStorage';
  serialize?: (value: T) => string;
  deserialize?: (value: string) => T;
  version?: number;
  migrate?: (oldValue: any, oldVersion: number) => T;
}

export function usePersistentState<T>(options: PersistentStateOptions<T>) {
  const {
    key,
    defaultValue,
    storage: storageType = 'localStorage',
    serialize: _serialize = JSON.stringify,
    deserialize: _deserialize = JSON.parse,
    version = 1,
    migrate
  } = options;

  const [state, setState] = useState<T>(() => {
    if (typeof window === 'undefined') {
      return defaultValue;
    }

    try {
      const item = storage.get(key, storageType);
      if (item === null) {
        return defaultValue;
      }

      // Handle versioning and migration
      if (version && migrate && item._version !== version) {
        const migrated = migrate(item, item._version || 0);
        const versionedState = { ...migrated, _version: version };
        storage.set(key, versionedState, storageType);
        return migrated;
      }

      return item._version ? item : item;
    } catch (error) {
      console.warn(`Failed to load persisted state for key "${key}":`, error);
      return defaultValue;
    }
  });

  // Save to storage when state changes
  useEffect(() => {
    if (typeof window === 'undefined') return;

    try {
      const valueToStore = version ? { ...state, _version: version } : state;
      storage.set(key, valueToStore, storageType);
    } catch (error) {
      console.warn(`Failed to persist state for key "${key}":`, error);
    }
  }, [state, key, storageType, version]);

  const updateState = useCallback((value: T | ((prev: T) => T)) => {
    setState(prevState => {
      const newState = typeof value === 'function' 
        ? (value as (prev: T) => T)(prevState)
        : value;
      return newState;
    });
  }, []);

  const resetState = useCallback(() => {
    setState(defaultValue);
    if (typeof window !== 'undefined') {
      storage.remove(key, storageType);
    }
  }, [defaultValue, key, storageType]);

  const clearStorage = useCallback(() => {
    if (typeof window !== 'undefined') {
      storage.remove(key, storageType);
    }
  }, [key, storageType]);

  return [state, updateState, { reset: resetState, clear: clearStorage }] as const;
}

// Specialized hooks for common use cases
export function useLocalStorage<T>(key: string, defaultValue: T) {
  return usePersistentState({
    key,
    defaultValue,
    storage: 'localStorage'
  });
}

export function useSessionStorage<T>(key: string, defaultValue: T) {
  return usePersistentState({
    key,
    defaultValue,
    storage: 'sessionStorage'
  });
}

// Hook for user preferences with type safety
interface UserPreferences {
  theme: 'light' | 'dark' | 'system';
  language: string;
  currency: string;
  notifications: {
    email: boolean;
    push: boolean;
    trading: boolean;
  };
  trading: {
    defaultView: 'grid' | 'list' | 'chart';
    autoRefresh: boolean;
    refreshInterval: number;
  };
}

const defaultPreferences: UserPreferences = {
  theme: 'system',
  language: 'en',
  currency: 'USD',
  notifications: {
    email: true,
    push: true,
    trading: true
  },
  trading: {
    defaultView: 'grid',
    autoRefresh: true,
    refreshInterval: 30000
  }
};

export function useUserPreferences() {
  return usePersistentState({
    key: 'epsx-user-preferences',
    defaultValue: defaultPreferences,
    version: 2,
    migrate: (oldPrefs: any, oldVersion: number) => {
      // Example migration from v1 to v2
      if (oldVersion < 2) {
        return {
          ...defaultPreferences,
          ...oldPrefs,
          trading: {
            ...defaultPreferences.trading,
            ...(oldPrefs.trading || {})
          }
        };
      }
      return oldPrefs;
    }
  });
}

// Hook for temporary state that persists across page reloads but not browser sessions
export function useTemporaryPersistentState<T>(key: string, defaultValue: T) {
  return usePersistentState({
    key: `temp-${key}`,
    defaultValue,
    storage: 'sessionStorage'
  });
}

// Hook for form state persistence
export function useFormPersistence<T extends Record<string, any>>(
  formId: string,
  initialValues: T,
  options: { 
    clearOnSubmit?: boolean;
    ttl?: number; // Time to live in milliseconds
  } = {}
) {
  const { clearOnSubmit = true, ttl } = options;
  
  const [formData, setFormData, { clear }] = useSessionStorage(
    `form-${formId}`,
    { values: initialValues, timestamp: Date.now() }
  );

  // Check if data has expired
  const isExpired = ttl && (Date.now() - formData.timestamp) > ttl;
  const currentValues = isExpired ? initialValues : formData.values;

  const updateField = useCallback((field: keyof T, value: any) => {
    setFormData(prev => ({
      values: { ...prev.values, [field]: value },
      timestamp: Date.now()
    }));
  }, [setFormData]);

  const updateFields = useCallback((fields: Partial<T>) => {
    setFormData(prev => ({
      values: { ...prev.values, ...fields },
      timestamp: Date.now()
    }));
  }, [setFormData]);

  const resetForm = useCallback(() => {
    setFormData({ values: initialValues, timestamp: Date.now() });
  }, [setFormData, initialValues]);

  const clearForm = useCallback(() => {
    clear();
  }, [clear]);

  const onSubmit = useCallback((callback: (values: T) => void | Promise<void>) => {
    return async () => {
      await callback(currentValues);
      if (clearOnSubmit) {
        clearForm();
      }
    };
  }, [currentValues, clearOnSubmit, clearForm]);

  return {
    values: currentValues,
    updateField,
    updateFields,
    resetForm,
    clearForm,
    onSubmit,
    isExpired
  };
}