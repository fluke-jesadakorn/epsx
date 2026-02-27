'use client';

import { AuthModal } from '@/shared/components/auth';
import { useRouter } from 'next/navigation';

export function FrontendAuthGate() {
    const router = useRouter();

    return (
        <div className="fixed inset-0 bg-background/90 backdrop-blur-sm z-50">
            <AuthModal
                isOpen={true}
                onClose={() => { router.back(); }}
                variant="user"
                onSuccess={() => { router.refresh(); }}
            />
        </div>
    );
}
