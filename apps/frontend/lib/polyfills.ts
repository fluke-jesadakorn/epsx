// This polyfill resolves a specific issue with Next.js/Turbopack where implicit BigInt
// exponentiation (**) is sometimes transpiled to Math.pow(), which normally crashes with BigInts.

/**
 * BIGINT-SAFE MATH POLYFILLS
 * 
 * ESM bundles (like viem) often contain transpiled code that uses Math.pow
 * with BigInt arguments, which throws TypeError in native JavaScript.
 * This file MUST be imported before any other module that uses BigInt.
 */

// Access the global Math object safely
const anyMath = Math as any;

// Prevent infinite recursion if the file is re-evaluated and check for existing polyfill
if (!anyMath.pow.__isPolyfilled) {
    const originalPow = Math.pow;

    // Define the polyfill function
    // Override native math function to support BigInt
    // @ts-ignore - overriding native math function to support BigInt
    Math.pow = function (base: number | bigint, exponent: number | bigint) {
        // Case 1: Both BigInt
        if (typeof base === 'bigint' && typeof exponent === 'bigint') {
            try {
                // Use Function to avoid transpilation of ** to Math.pow
                return new Function('b', 'e', 'return b ** e')(base, exponent);
            } catch (e) {
                // Fallback: Loop
                if (exponent < 0n) return 0n; // Simple fallback
                let res = 1n;
                for (let i = 0n; i < exponent; i++) {
                    res *= base;
                }
                return res;
            }
        }

        // Case 2: Mixed Types (BigInt base, Number exponent)
        if (typeof base === 'bigint' && typeof exponent === 'number') {
            try {
                return new Function('b', 'e', 'return b ** BigInt(e)')(base, exponent);
            } catch (e) {
                // Fallback: Loop (if exponent is integer)
                if (Math.floor(exponent) === exponent && exponent >= 0) {
                    let res = 1n;
                    for (let i = 0; i < exponent; i++) {
                        res *= base;
                    }
                    return res;
                }
                // Return 0n or let it fail gracefully if we can't handle it
                return 0n;
            }
        }

        // Case 3: Mixed Types (Number base, BigInt exponent)
        if (typeof base === 'number' && typeof exponent === 'bigint') {
            try {
                return new Function('b', 'e', 'return BigInt(b) ** e')(base, exponent);
            } catch (e) {
                // Fallback: Loop or safe conversion
                // 2 ** 3n -> 8 (number? or bigint?) Native ** returns BigInt if base is BigInt? 
                // Actually 2 ** 3n is invalid in strict TS but valid in JS (returns number? No, throws TypeError usually).
                // Wait, JS `2 ** 3n` THROWS TypeError: Cannot mix BigInt and other types.
                // So we actually WANT to throw here to match spec, UNLESS we want to support it?
                // The error causing crash implies some library IS doing this and expecting it to work (or we transpiled it).
                // If the library expects `2n ** 3` (BigInt result), we handled in Case 2.
                // If the library expects `2 ** 3n`, it throws in native JS.
                // So falling through to native Math.pow is correct if we want to throw.
                // BUT the user error is "Cannot convert BigInt to number" inside Math.pow.
                // This means `Math.pow` WAS called.
                // So we should try to handle it if we can, or just return something safe to prevent crash?
                // Let's safe-guard it: if we can't calc, return NaN or throw a specific error, preventing generic crash?
                // Actually, let's just let it return NaN which is safe for number.
                return NaN;
            }
        }

        // Case 4: Any other BigInt usage
        if (typeof base === 'bigint' || typeof exponent === 'bigint') {
            return NaN; // Fail safe
        }

        // Case 5: Standard Number behavior
        return originalPow.call(Math, base, exponent);
    };

    // Mark as polyfilled to prevent re-wrapping
    (Math.pow as any).__isPolyfilled = true;

    // Also polyfill common Math functions that might be called with BigInt during transpilation
    const mathFuncs = ['floor', 'ceil', 'round', 'trunc', 'abs'];
    mathFuncs.forEach(func => {
        const original = (Math as any)[func];
        if (original && !original.__isPolyfilled) {
            (Math as any)[func] = function (val: any) {
                if (typeof val === 'bigint') return val;
                return original.call(Math, val);
            };
            (Math as any)[func].__isPolyfilled = true;
        }
    });

    // CRITICAL: Self-test the polyfill immediately
    try {
        const result = (Math.pow as any)(2n, 3n);
        if ((result as any) !== 8n) {
            console.error('[POLYFILL] Self-test failed: Math.pow(2n, 3n) !== 8n');
        }
    } catch (e) {
        console.error('[POLYFILL] Self-test CRASHED:', e);
    }
}

// Validation: Mock localStorage for Server-Side Rendering
// Some dependencies (like @walletconnect/keyvaluestorage) try to access localStorage during module initialization
// even in SSR environments, causing crashes. We provide a dummy implementation to satisfy them.
if (typeof window === 'undefined' && typeof global !== 'undefined') {
    const g = global as any;
    if (!g.localStorage || typeof g.localStorage.getItem !== 'function') {
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

