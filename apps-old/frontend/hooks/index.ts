'use client';

import { useCallback, useEffect, useRef, useState } from 'react';

// DEPRECATED: Generic data fetching hook using deprecated API client
// Migrate to specific server actions instead of using this hook
export function useApi<T>(
  endpoint: string,
  options?: { refresh?: number; enabled?: boolean }
) {
  const [data, _setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = useCallback(() => {
    if (options?.enabled === false) {return;}

    setError('useApi hook is deprecated. Please use specific server actions instead.');
    setLoading(false);
  }, [options?.enabled]);

  useEffect(() => {
    fetchData();

    if (options?.refresh != null && options.refresh > 0) {
      const interval = setInterval(() => fetchData(), options.refresh);
      return () => clearInterval(interval);
    }
    return undefined;
  }, [fetchData, options?.refresh]);

  return { data, loading, error, refetch: fetchData };
}

// Form handling hook
export function useForm<T extends Record<string, unknown>>(
  initialValues: T,
  onSubmit: (values: T) => Promise<void>
) {
  const [values, setValues] = useState<T>(initialValues);
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState<Partial<Record<keyof T, string>>>({});
  const [formError, setFormError] = useState<string | null>(null);

  const handleChange = (name: keyof T, value: unknown) => {
    setValues(prev => ({ ...prev, [name]: value }));
    setErrors(prev => ({ ...prev, [name]: undefined }));
    setFormError(null);
  };

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault();
    setLoading(true);
    setErrors({});
    setFormError(null);

    try {
      await onSubmit(values);
    } catch (err) {
      setFormError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const reset = () => {
    setValues(initialValues);
    setErrors({});
    setFormError(null);
  };

  return {
    values,
    loading,
    errors,
    formError,
    handleChange,
    handleSubmit,
    reset,
    setValues,
  };
}

// Debounced value hook (different from utility debounce function)
export function useDebounce<T>(value: T, delay: number): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(handler);
    };
  }, [value, delay]);

  return debouncedValue;
}

// Cookie storage hook (replaces localStorage)
export function useCookieStorage<T>(key: string, initialValue: T, maxAge?: number) {
  const [storedValue, setStoredValue] = useState<T>(() => {
    if (typeof window === 'undefined') {return initialValue;}

    try {
      // Try cookie first, then fallback to localStorage for migration
      const cookies = document.cookie.split(';').reduce<Record<string, string>>((acc, cookie) => {
        const [k, v] = cookie.trim().split('=');
        if (k && v) {acc[k] = v;}
        return acc;
      }, {});

      const item = cookies[key] ?? window.localStorage.getItem(key);
      return (item !== null && item !== undefined && item !== '') ? (JSON.parse(item) as T) : initialValue;
    } catch (_error) {
      return initialValue;
    }
  });

  const setValue = (value: T | ((val: T) => T)) => {
    try {
      const valueToStore = value instanceof Function ? value(storedValue) : value;
      setStoredValue(valueToStore);
      if (typeof window !== 'undefined') {
        // Store in cookie instead of localStorage
        const maxAgeStr = (maxAge != null && maxAge > 0) ? `max-age=${maxAge}` : '';
        document.cookie = `${key}=${encodeURIComponent(JSON.stringify(valueToStore))}; path=/; ${maxAgeStr} SameSite=lax`;
      }
    } catch (_error) {
      // Cookie setting failed silently
    }
  };

  return [storedValue, setValue] as const;
}

// Deprecated localStorage hook (for migration only - useCookieStorage instead)
export function useLocalStorage<T>(key: string, initialValue: T) {
  return useCookieStorage(key, initialValue);
}

// Media query hook
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(false);

  useEffect(() => {
    const media = window.matchMedia(query);
    setMatches(media.matches);

    const listener = () => setMatches(media.matches);
    media.addEventListener('change', listener);

    return () => media.removeEventListener('change', listener);
  }, [query]);

  return matches;
}

// Click outside hook
export function useClickOutside<T extends HTMLElement>(
  callback: () => void
) {
  const ref = useRef<T>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        callback();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [callback]);

  return ref;
}

