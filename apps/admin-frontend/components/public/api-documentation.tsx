'use client';

import React from 'react';
import {
    ApiHeader,
    AuthenticationSection,
    BaseUrlSection,
    ErrorCodesSection,
    ModulesList,
    RateLimitsSection,
    SupportSection,
} from './api-documentation-ui';
import {
    useAccessLevelColor,
    useCopyToClipboard,
    useDocumentationExpanded,
    useMethodColor,
} from './api-documentation-hooks';
import { moduleDocumentation } from './api-documentation-data';

export const ApiDocumentation: React.FC = () => {
    const { expandedModule, expandedEndpoint, toggleModule, toggleEndpoint } = useDocumentationExpanded();
    const { copyToClipboard } = useCopyToClipboard();
    const getMethodColor = useMethodColor();
    const getAccessLevelColor = useAccessLevelColor();

    return (
        <div className="max-w-6xl mx-auto p-6">
            <ApiHeader onRequestAccess={() => void 0} />

            <AuthenticationSection onCopy={copyToClipboard} />

            <BaseUrlSection onCopy={copyToClipboard} />

            <RateLimitsSection />

            <ModulesList
                modules={moduleDocumentation}
                expandedModule={expandedModule}
                expandedEndpoint={expandedEndpoint}
                onToggleModule={toggleModule}
                onToggleEndpoint={toggleEndpoint}
                onCopy={copyToClipboard}
                getMethodColor={getMethodColor}
                getAccessLevelColor={getAccessLevelColor}
            />

            <ErrorCodesSection />

            <SupportSection />
        </div>
    );
};
