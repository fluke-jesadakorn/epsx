'use client';

import type { ReactNode } from 'react';

interface AnalyticsAuthWrapperProps {
    children: ReactNode;
}

export function AnalyticsAuthWrapper({ children }: AnalyticsAuthWrapperProps) {
    return <>{children}</>;
}
