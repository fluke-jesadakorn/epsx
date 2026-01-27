'use client';

import { ReactNode } from 'react';
import { FrontendAuthModal } from './FrontendAuthModal';

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
