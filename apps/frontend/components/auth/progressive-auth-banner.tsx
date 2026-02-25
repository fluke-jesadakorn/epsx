'use client';

import { AuthBanner } from '@/shared/components/auth/auth-banner';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';

interface ProgressiveAuthBannerProps {
    message?: string;
}

export function ProgressiveAuthBanner({
    message = 'Sign in to unlock full access'
}: ProgressiveAuthBannerProps) {
    const router = useRouter();
    const pathname = usePathname();
    const searchParams = useSearchParams();

    const handleSignIn = () => {
        const returnUrl = searchParams.toString()
            ? `${pathname}?${searchParams.toString()}`
            : pathname;
        router.push(`/auth?return_url=${encodeURIComponent(returnUrl)}`);
    };

    return <AuthBanner message={message} onSignIn={handleSignIn} />;
}
