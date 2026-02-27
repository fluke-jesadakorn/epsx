'use client';

import { AuthBanner } from '@/shared/components/auth/auth-banner';

interface ProgressiveAuthBannerProps {
    message?: string;
    description?: string;
}

// Uses openSignInModal from AuthBanner (via useSharedAuth) - no redirect needed
export function ProgressiveAuthBanner({
    message,
    description,
}: ProgressiveAuthBannerProps) {
    return <AuthBanner message={message} description={description} />;
}
