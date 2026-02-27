'use client';

import { Turnstile } from '@marsidev/react-turnstile';
import { useTheme } from 'next-themes';
import { useEffect, useState } from 'react';

/**
 * Reusable Cloudflare Turnstile CAPTCHA widget
 */
export interface TurnstileWidgetProps {
    onSuccess: (token: string) => void;
    onError?: () => void;
    onExpire?: () => void;
    className?: string;
    action?: string;
}

export function TurnstileWidget({
    onSuccess,
    onError,
    onExpire,
    className,
    action = 'login',
}: TurnstileWidgetProps) {
    const { resolvedTheme } = useTheme();
    const [siteKey, setSiteKey] = useState<string | null>(null);

    useEffect(() => {
        // Skip in development regardless of site key (Cloudflare unreachable on local IPs)
        if (process.env.NODE_ENV === 'development') {
            onSuccess('development-skip-token');
            return;
        }
        const key = process.env.NEXT_PUBLIC_TURNSTILE_SITE_KEY;
        if (key !== undefined && key !== '') {
            setSiteKey(key);
        } else {
            onSuccess('development-skip-token');
        }
    }, [onSuccess]);

    if (siteKey === null) {
        return null;
    }

    return (
        <div className={`flex justify-center ${className ?? ''}`}>
            <Turnstile
                siteKey={siteKey}
                options={{
                    action,
                    theme: resolvedTheme === 'dark' ? 'dark' : 'light',
                }}
                onSuccess={onSuccess}
                onError={onError}
                onExpire={onExpire}
            />
        </div>
    );
}
