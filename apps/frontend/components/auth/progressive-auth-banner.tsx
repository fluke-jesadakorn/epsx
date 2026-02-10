'use client';

import { AuthBanner } from '@/shared/components/auth/auth-banner';
import { usePathname, useRouter } from 'next/navigation';

interface ProgressiveAuthBannerProps {
    message?: string;
}

export function ProgressiveAuthBanner({
    message = 'Sign in to unlock full access'
}: ProgressiveAuthBannerProps) {
    const router = useRouter();
    const pathname = usePathname();

    const handleSignIn = () => {
        router.push(`/auth?return_url=${encodeURIComponent(pathname)}`);
    };

    return <AuthBanner message={message} onSignIn={handleSignIn} />;
}
