'use client';

import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/Provider';

export function FrontendAuthModal() {
    const { isAuthenticated, showSignInModal, closeSignInModal } = useSharedAuth();

    if (isAuthenticated) return null;

    return (
        <AuthModal
            isOpen={showSignInModal}
            onClose={closeSignInModal}
            variant="user"
            onSuccess={closeSignInModal}
        />
    );
}
