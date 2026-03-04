'use client';

import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/provider';

export function FrontendAuthModal() {
    const { showSignInModal, closeSignInModal } = useSharedAuth();

    return (
        <AuthModal
            isOpen={showSignInModal}
            onClose={closeSignInModal}
            variant="user"
            onSuccess={closeSignInModal}
        />
    );
}
