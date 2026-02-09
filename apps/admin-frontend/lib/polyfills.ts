// This polyfill resolves a specific issue with Next.js/Turbopack where implicit BigInt
// exponentiation (**) is sometimes transpiled to Math.pow(), which normally crashes with BigInts.

/**
 * Interface for polyfilled Math.pow
 */
interface PolyfilledPow {
    (base: number | bigint, exponent: number | bigint): number | bigint;
    __isPolyfilled?: boolean;
}

// Access the global Math object safely
const math = Math as unknown as { pow: PolyfilledPow };

/**
 * Helper to perform BigInt exponentiation using dynamic evaluation
 * to bypass potential transpiler issues.
 */
function bigIntPow(base: bigint, exponent: number | bigint): bigint {
    try {
        // Use Function constructor to bypass transpiler math.pow replacement
        const evaluator = new Function('b', 'e', 'return b ** e') as (b: bigint, e: number | bigint) => bigint;
        return evaluator(base, exponent);
    } catch (_e) {
        // Fallback: Loop for positive integer exponents
        const exponentBig = typeof exponent === 'number' ? BigInt(exponent) : exponent;
        if (exponentBig < 0n) {
            return 0n;
        }

        let res = 1n;
        for (let i = 0n; i < exponentBig; i++) {
            res *= base;
        }
        return res;
    }
}

/**
 * The polyfill implementation for Math.pow
 */
function createPolyfilledPow(originalPow: (x: number, y: number) => number): PolyfilledPow {
    return function (base: number | bigint, exponent: number | bigint): number | bigint {
        // Case 1 & 2: BigInt base
        if (typeof base === 'bigint') {
            return bigIntPow(base, exponent);
        }

        // Case 3: Mixed Types (Number base, BigInt exponent)
        if (typeof exponent === 'bigint') {
            return NaN; // BigInt exponent with Number base is invalid in JS
        }

        // Case 5: Standard Number behavior
        return originalPow.call(Math, base, exponent);
    };
}

// Prevent infinite recursion if the file is re-evaluated and check for existing polyfill
if (math.pow.__isPolyfilled !== true) {
    const originalPow = Math.pow;
    const polyfilledPow = createPolyfilledPow(originalPow);

    // Mark as polyfilled to prevent re-wrapping
    polyfilledPow.__isPolyfilled = true;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (Math as unknown as Record<string, any>).pow = polyfilledPow;
}

// Validation: Mock localStorage for Server-Side Rendering
if (typeof window === 'undefined' && typeof global !== 'undefined') {
    const g = global as unknown as { localStorage?: Record<string, unknown> };
    const storage = g.localStorage;
    const needsLocalStorage = !storage || typeof storage.getItem !== 'function';

    if (needsLocalStorage) {
        g.localStorage = {
            getItem: () => null,
            setItem: () => { },
            removeItem: () => { },
            clear: () => { },
            key: () => null,
            length: 0,
        };
    }
}

export { };
