'use client';

import Script from 'next/script';
import { useTheme } from 'next-themes';
import { useEffect, useRef, useState } from 'react';

declare global {
    interface Window {
        Scalar?: {
            createApiReference: (el: HTMLElement | string, config: Record<string, unknown>) => void;
        };
    }
}

export default function DeveloperDocumentationPage() {
    const { resolvedTheme } = useTheme();
    const isDark = resolvedTheme !== 'light';
    const containerRef = useRef<HTMLDivElement>(null);
    const [scriptLoaded, setScriptLoaded] = useState(false);

    useEffect(() => {
        if (!scriptLoaded || !containerRef.current || !window.Scalar) return;
        const el = containerRef.current;
        el.innerHTML = '';

        const isDev = process.env.NODE_ENV !== 'production';
        const host = window.location.hostname;
        const servers: { url: string; description: string }[] = [
            { url: 'http://localhost:8080', description: 'Local' },
        ];
        if (isDev && host !== 'localhost' && host !== '127.0.0.1') {
            servers.push({ url: `http://${host}:8080`, description: `Dev (${host})` });
        }
        servers.push({ url: 'https://api.epsx.io', description: 'Production' });

        window.Scalar.createApiReference(el, {
            url: '/developer/docs/openapi',
            darkMode: isDark,
            theme: isDark ? 'deepSpace' : 'default',
            layout: 'modern',
            showSidebar: true,
            searchHotKey: 'k',
            authentication: { preferredSecurityScheme: 'bearerAuth' },
            servers,
            defaultHttpClient: { targetKey: 'javascript', clientKey: 'fetch' },
        });
    }, [scriptLoaded, isDark]);

    return (
        // Break out of developer layout padding with negative margins
        <div className="-mx-4 lg:-mx-8 -mt-28 lg:-mt-8 -mb-6 lg:-mb-8" style={{ height: '100svh' }}>
            <div ref={containerRef} style={{ height: '100%', overflow: 'auto' }} />
            <Script
                src="/scalar-api-reference.js"
                strategy="afterInteractive"
                onLoad={() => setScriptLoaded(true)}
            />
        </div>
    );
}
