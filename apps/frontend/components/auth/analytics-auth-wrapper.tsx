'use client';

import type { ReactNode } from 'react';
import { FrontendAuthModal } from './frontend-auth-modal';

interface AnalyticsAuthWrapperProps {
    children: ReactNode;
}

export function AnalyticsAuthWrapper({ children }: AnalyticsAuthWrapperProps) {
    return (
        <>
            {children}
            <FrontendAuthModal />
        </>
    );
}
