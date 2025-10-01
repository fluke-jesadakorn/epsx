// Browser API polyfills for SSR compatibility

// IndexedDB polyfill for server-side rendering
if (typeof globalThis !== 'undefined' && typeof globalThis.indexedDB === 'undefined') {
  // Create a minimal IndexedDB mock for SSR with proper error handling
  const mockIndexedDB = {
    open: () => {
      const mockDB = {
        onclose: null,
        onerror: null,
        onversionchange: null,
        transaction: () => {
          const mockTransaction = {
            oncomplete: null,
            onerror: null,
            onabort: null,
            objectStore: () => ({
              get: () => ({ onsuccess: null, onerror: null, result: null }),
              put: () => ({ onsuccess: null, onerror: null }),
              delete: () => ({ onsuccess: null, onerror: null }),
              clear: () => ({ onsuccess: null, onerror: null }),
              count: () => ({ onsuccess: null, onerror: null, result: 0 }),
            })
          };
          return mockTransaction;
        },
        close: () => {},
        createObjectStore: () => ({}),
        deleteObjectStore: () => {},
      };

      const mockRequest = {
        onsuccess: null as ((event: any) => void) | null,
        onerror: null as ((event: any) => void) | null,
        result: mockDB,
        error: null,
        readyState: 'done',
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      };

      // Simulate async success with proper error handling
      setTimeout(() => {
        try {
          if (mockRequest.onsuccess) {
            mockRequest.onsuccess({ target: mockRequest } as any);
          }
        } catch (error) {
          // Silently handle mock database initialization errors
        }
      }, 0);
      return mockRequest;
    },
    deleteDatabase: () => ({
      onsuccess: null,
      onerror: null,
    }),
    cmp: () => 0,
  };

  // @ts-ignore
  globalThis.indexedDB = mockIndexedDB;
  
  // Also add to global for Node.js environments
  if (typeof global !== 'undefined') {
    // @ts-ignore
    global.indexedDB = mockIndexedDB;
  }
}

// Additional browser API polyfills if needed
if (typeof globalThis !== 'undefined') {
  if (typeof globalThis.localStorage === 'undefined') {
    // @ts-ignore
    globalThis.localStorage = {
      getItem: () => null,
      setItem: () => {},
      removeItem: () => {},
      clear: () => {},
      length: 0,
      key: () => null,
    };
  }

  if (typeof globalThis.sessionStorage === 'undefined') {
    // @ts-ignore
    globalThis.sessionStorage = {
      getItem: () => null,
      setItem: () => {},
      removeItem: () => {},
      clear: () => {},
      length: 0,
      key: () => null,
    };
  }
}

export {};