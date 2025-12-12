// Async Storage stub for Metamask SDK compatibility
// This provides a minimal implementation for SSR compatibility

export const AsyncStorage = {
  getItem: async (key: string): Promise<string | null> => {
    if (typeof window !== 'undefined' && window.localStorage) {
      return window.localStorage.getItem(key);
    }
    return null;
  },

  setItem: async (key: string, value: string): Promise<void> => {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.setItem(key, value);
    }
  },

  removeItem: async (key: string): Promise<void> => {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.removeItem(key);
    }
  },

  clear: async (): Promise<void> => {
    if (typeof window !== 'undefined' && window.localStorage) {
      window.localStorage.clear();
    }
  },

  getAllKeys: async (): Promise<string[]> => {
    if (typeof window !== 'undefined' && window.localStorage) {
      return Object.keys(window.localStorage);
    }
    return [];
  },

  multiGet: async (keys: string[]): Promise<[string, string | null][]> => {
    const result: [string, string | null][] = [];
    for (const key of keys) {
      const value = await AsyncStorage.getItem(key);
      result.push([key, value]);
    }
    return result;
  },

  multiSet: async (keyValuePairs: [string, string][]): Promise<void> => {
    for (const [key, value] of keyValuePairs) {
      await AsyncStorage.setItem(key, value);
    }
  },

  multiRemove: async (keys: string[]): Promise<void> => {
    for (const key of keys) {
      await AsyncStorage.removeItem(key);
    }
  }
};

export default AsyncStorage;