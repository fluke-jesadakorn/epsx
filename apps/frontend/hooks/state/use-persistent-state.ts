import { useState, useEffect, useCallback } from 'react';

interface VersionedData {
  _version?: number;
  [key: string]: unknown;
}

// Storage utilities (inlined from lib/state/store.ts)
const storage = {
  get: (key: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage'): unknown | null => {
    if (typeof window === 'undefined') {
      return null;
    }
    try {
      const item = window[storageType].getItem(key);
      if (item !== null && item !== '') {
        return JSON.parse(item) as unknown;
      }
      return null;
    } catch {
      return null;
    }
  },

  set: (key: string, value: unknown, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') {return;}
    try {
      window[storageType].setItem(key, JSON.stringify(value));
    } catch (_error) {
      // Storage write failed silently
    }
  },

  remove: (key: string, storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') {return;}
    try {
      window[storageType].removeItem(key);
    } catch (_error) {
      // Storage removal failed silently
    }
  },

  clear: (storageType: 'localStorage' | 'sessionStorage' = 'localStorage') => {
    if (typeof window === 'undefined') {return;}
    try {
      window[storageType].clear();
    } catch (_error) {
      // Storage clear failed silently
    }
  }
};

interface PersistentStateOptions<T> {
  key: string;
  defaultValue: T;
  storage?: 'localStorage' | 'sessionStorage';
  serialize?: (value: T) => string;
  deserialize?: (value: string) => T;
  version?: number;
  migrate?: (oldValue: unknown, oldVersion: number) => T;
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
      const item: unknown = storage.get(key, storageType);
      if (item === null) {
        return defaultValue;
      }

      // Type guard for versioned data
      const isVersionedData = (value: unknown): value is VersionedData => {
        return typeof value === 'object' && value !== null && '_version' in value;
      };

      // Handle versioning and migration
      if (migrate !== undefined && isVersionedData(item)) {
        const itemVersion = typeof item._version === 'number' ? item._version : 0;
        if (itemVersion !== version) {
          const migrated = migrate(item, itemVersion);
          const versionedState: VersionedData = { ...(migrated as Record<string, unknown>), _version: version };
          storage.set(key, versionedState, storageType);
          return migrated;
        }
      }

      return isVersionedData(item) && item._version !== undefined ? (item as T) : (item as T);
    } catch (_error) {
      return defaultValue;
    }
  });

  // Save to storage when state changes
  useEffect(() => {
    if (typeof window === 'undefined') {return;}

    try {
      const valueToStore = version ? { ...state, _version: version } : state;
      storage.set(key, valueToStore, storageType);
    } catch (_error) {
      // Persistence failed silently
    }
  }, [state, key, storageType, version]);

  const updateState = useCallback((value: T | ((prev: T) => T)) => {
    setState(prevState => {
      return typeof value === 'function'
        ? (value as (prev: T) => T)(prevState)
        : value;
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
    migrate: (oldPrefs: unknown, oldVersion: number) => {
      // Example migration from v1 to v2
      if (oldVersion < 2) {
        const oldPrefObj = oldPrefs as Record<string, unknown> | null;
        const safeOldPrefObj = oldPrefObj ?? {};
        const oldTrading = typeof safeOldPrefObj.trading === 'object' && safeOldPrefObj.trading !== null ? safeOldPrefObj.trading as Record<string, unknown> : {};
        return {
          ...defaultPreferences,
          ...safeOldPrefObj,
          trading: {
            ...defaultPreferences.trading,
            ...oldTrading
          }
        };
      }
      return (oldPrefs as UserPreferences | null) !== null ? (oldPrefs as UserPreferences) : defaultPreferences;
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
export function useFormPersistence<T extends Record<string, unknown>>(
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
  const isExpired = ttl !== undefined && ttl !== 0 && !Number.isNaN(ttl) && (Date.now() - formData.timestamp) > ttl;
  const currentValues = (isExpired === true) ? initialValues : formData.values;

  const updateField = useCallback((field: keyof T, value: unknown) => {
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