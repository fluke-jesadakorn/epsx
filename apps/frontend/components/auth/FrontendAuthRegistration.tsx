/**
 * Frontend Auth Registration
 * 
 * Registers the frontend auth hook with the unified auth adapter.
 * This should be imported early in the app to ensure the hook is available.
 */
'use client';

import { useFrontendAuth } from '@/hooks/useFrontendAuth';
import { registerAuthHook } from '@/shared/components/auth/UnifiedAuthAdapter';
import { useEffect } from 'react';

let isRegistered = false;

/**
 * Component that registers the frontend auth hook
 * Include this once in your app's provider tree
 */
export function FrontendAuthRegistration({ children }: { children: React.ReactNode }) {
    useEffect(() => {
        if (!isRegistered) {
            registerAuthHook('frontend', useFrontendAuth);
            isRegistered = true;
        }
    }, []);

    return <>{children}</>;
}

/**
 * Register the frontend auth hook immediately (for non-component contexts)
 */
export function ensureFrontendAuthRegistered() {
    if (!isRegistered) {
        registerAuthHook('frontend', useFrontendAuth);
        isRegistered = true;
    }
}

export default FrontendAuthRegistration;
