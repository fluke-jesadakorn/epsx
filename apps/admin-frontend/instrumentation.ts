/**
 * Next.js Instrumentation Hook
 * This file runs very early in the Next.js lifecycle, before other modules are loaded.
 * Used to apply critical polyfills that must be in place before any library code runs.
 */

// Apply BigInt safety polyfill for Math.pow IMMEDIATELY
// This fixes "Cannot convert a BigInt value to a number" errors from wagmi/viem/rainbowkit
const originalPow = Math.pow;
Math.pow = function safePow(base: number | bigint, exponent: number | bigint): number {
    const safeBase = typeof base === 'bigint' ? Number(base) : base;
    const safeExponent = typeof exponent === 'bigint' ? Number(exponent) : exponent;
    return originalPow(safeBase, safeExponent);
};

/**
 *
 */
export async function register() {
    // The polyfill is already applied at module load time above
    // This function is required for Next.js instrumentation hook

    if (process.env.NEXT_RUNTIME === 'nodejs') {
        // Server-side specific initialization can go here
    }

    if (process.env.NEXT_RUNTIME === 'edge') {
        // Edge runtime specific initialization can go here
    }
}

// Also export for client-side if needed
/**
 *
 */
export function onRequestError() {
    // Optional: handle request errors
}
