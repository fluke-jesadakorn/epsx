import { ApiReference } from '@scalar/nextjs-api-reference';
import type { NextRequest } from 'next/server';

const DEV_SERVER = { url: 'http://localhost:8080', description: 'Development Server' };
const PROD_SERVER = { url: 'https://api.epsx.io', description: 'Production Server' };

function handler(req: NextRequest): Response {
    const dark = req.nextUrl.searchParams.get('dark') !== 'false';
    const isProd = (process.env.NEXT_PUBLIC_BACKEND_URL ?? '').includes('api.epsx.io');
    const servers = isProd ? [PROD_SERVER, DEV_SERVER] : [DEV_SERVER, PROD_SERVER];

    return ApiReference({
        cdn: '/scalar-api-reference.js',
        url: '/developer/docs/openapi',
        darkMode: dark,
        theme: dark ? 'deepSpace' : 'default',
        layout: 'modern',
        showSidebar: true,
        searchHotKey: 'k',
        authentication: {
            preferredSecurityScheme: 'bearerAuth',
        },
        servers,
        defaultHttpClient: {
            targetKey: 'javascript',
            clientKey: 'fetch',
        },
    })();
}

export const GET = handler;
