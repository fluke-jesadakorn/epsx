// Browser API polyfills for SSR compatibility

// IndexedDB polyfill for server-side rendering
if (typeof globalThis !== 'undefined' && typeof globalThis.indexedDB === 'undefined') {
  // Create a minimal IndexedDB mock for SSR
  const mockIndexedDB = {
    open: () => {
      const mockRequest = {
        onsuccess: null,
        onerror: null,
        result: null,
        error: null,
        readyState: 'done',
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      };
      // Simulate async success
      setTimeout(() => {
        if (mockRequest.onsuccess) {
          mockRequest.onsuccess({ target: mockRequest } as any);
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