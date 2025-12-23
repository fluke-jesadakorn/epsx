/**
 * Browser API polyfills for SSR compatibility
 * Shared across frontend applications
 */

// Marker to track if Math.pow has been patched
declare global {
    interface Math {
        __bigintSafe?: boolean;
    }
}

// Patch Math.pow to safely handle BigInt values (fixes wagmi/viem initialization issues)
// Only patch if not already patched (instrumentation.ts may have done this earlier)
if (typeof globalThis !== 'undefined' && typeof globalThis.Math !== 'undefined' && !Math.__bigintSafe) {
    const originalPow = Math.pow;
    Math.pow = function safePow(base: number | bigint, exponent: number | bigint): number {
        // Convert BigInt to Number if present
        const safeBase = typeof base === 'bigint' ? Number(base) : base;
        const safeExponent = typeof exponent === 'bigint' ? Number(exponent) : exponent;
        return originalPow(safeBase, safeExponent);
    };
    // Mark as patched
    Math.__bigintSafe = true;
}

// Enable BigInt JSON serialization to prevent "Cannot mix BigInt" errors
// This is a common issue with wagmi/viem that use BigInt extensively
declare global {
    interface BigInt {
        toJSON(): string;
    }
}

if (typeof BigInt !== 'undefined' && !BigInt.prototype.toJSON) {
    BigInt.prototype.toJSON = function () {
        return this.toString();
    };
}

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
                close: () => { },
                createObjectStore: () => ({}),
                deleteObjectStore: () => { },
            };

            const mockRequest = {
                onsuccess: null as ((event: unknown) => void) | null,
                onerror: null as ((event: unknown) => void) | null,
                result: mockDB,
                error: null,
                readyState: 'done',
                addEventListener: () => { },
                removeEventListener: () => { },
                dispatchEvent: () => false,
            };

            // Simulate async success with proper error handling
            setTimeout(() => {
                try {
                    if (mockRequest.onsuccess) {
                        mockRequest.onsuccess({ target: mockRequest });
                    }
                } catch {
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

    // @ts-expect-error - Adding polyfill to globalThis
    globalThis.indexedDB = mockIndexedDB;

    // Also add to global for Node.js environments
    if (typeof global !== 'undefined') {
        // @ts-expect-error - Adding polyfill to global
        global.indexedDB = mockIndexedDB;
    }
}

// Additional browser API polyfills if needed
if (typeof globalThis !== 'undefined') {
    if (typeof globalThis.localStorage === 'undefined') {
        globalThis.localStorage = {
            getItem: () => null,
            setItem: () => { },
            removeItem: () => { },
            clear: () => { },
            length: 0,
            key: () => null,
        };
    }

    if (typeof globalThis.sessionStorage === 'undefined') {
        globalThis.sessionStorage = {
            getItem: () => null,
            setItem: () => { },
            removeItem: () => { },
            clear: () => { },
            length: 0,
            key: () => null,
        };
    }
}

export { };
