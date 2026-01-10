// This polyfill resolves a specific issue with Next.js/Turbopack where implicit BigInt
// exponentiation (**) is sometimes transpiled to Math.pow(), which normally crashes with BigInts.

// Access the global Math object safely
const anyMath = Math as any;

// Prevent infinite recursion if the file is re-evaluated and check for existing polyfill
if (!anyMath.pow.__isPolyfilled) {
    const originalPow = Math.pow;

    // Define the polyfill function
    // @ts-expect-error - overriding native math function to support BigInt
    Math.pow = function (base: number | bigint, exponent: number | bigint) {
        if (typeof base === 'bigint' && typeof exponent === 'bigint') {
            // CRITICAL: We use new Function() here to bypass the transpiler.
            // If we used `base ** exponent` directly, Turbopack/SWC might transpile it back to 
            // `Math.pow(base, exponent)`, causing infinite recursion (Maximum call stack size exceeded).
            try {
                return new Function('b', 'e', 'return b ** e')(base, exponent);
            } catch (e) {
                // Fallback for CSP environments where new Function is blocked
                console.error('BigInt Math.pow polyfill failed (CSP blocked new Function?)', e);
                // Simple inefficient fallback just in case
                let res = 1n;
                for (let i = 0n; i < exponent; i++) {
                    res *= base;
                }
                return res;
            }
        }
        // @ts-expect-error - calling original with matched types
        return originalPow.call(Math, base, exponent);
    };

    // Mark as polyfilled to prevent re-wrapping
    (Math.pow as any).__isPolyfilled = true;
}

export { };

