
import { COOKIES } from '@/shared/auth/cookies';
import { createFrontendApiClient } from '@/shared/utils/api-client';
import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

// Match all HTTP methods
export async function GET(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    return handleProxy(req, ctx);
}

export async function POST(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    return handleProxy(req, ctx);
}

export async function PUT(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    return handleProxy(req, ctx);
}

export async function DELETE(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    return handleProxy(req, ctx);
}

export async function PATCH(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    return handleProxy(req, ctx);
}

async function handleProxy(req: NextRequest, ctx: { params: Promise<{ path: string[] }> }) {
    try {
        const { path } = await ctx.params;
        const cookieStore = await cookies();
        const token = cookieStore.get(COOKIES.access_token)?.value;

        // Construct backend URL path
        // path is an array like ['api', 'admin', 'wallets', 'stats']
        const targetPath = '/' + path.join('/');

        console.log(`🔄 API Proxy (Frontend): ${req.method} ${targetPath}`, {
            hasToken: !!token
        });

        // Initialize Unified Client (Server Side)
        const client = createFrontendApiClient({ serverSide: true });

        // Create headers to forward
        const forwardHeaders = new Headers();

        // Copy safe headers
        const allowedHeaders = ['accept', 'content-type', 'user-agent', 'x-forwarded-for', 'x-forwarded-proto', 'origin', 'referer'];
        req.headers.forEach((value, key) => {
            if (allowedHeaders.includes(key.toLowerCase()) || key.startsWith('x-')) {
                forwardHeaders.set(key, value);
            }
        });

        // Ensure we don't send cookies to backend
        forwardHeaders.delete('cookie');

        // Prepare body (for non-GET/HEAD requests)
        let body: any = (req.method !== 'GET' && req.method !== 'HEAD')
            ? await req.blob()
            : undefined;

        // Special handling for token refresh: inject refresh token from HttpOnly cookie
        if (targetPath === '/api/auth/session/refresh' && req.method === 'POST') {
            const refreshToken = cookieStore.get(COOKIES.refresh_token)?.value;
            if (refreshToken) {
                console.log('🔄 API Proxy: Injecting refresh token from cookie');
                body = JSON.stringify({ refresh_token: refreshToken });
                forwardHeaders.set('content-type', 'application/json');
            } else {
                console.warn('⚠️ API Proxy: No refresh token cookie found for refresh request');
            }
        }

        // Use Unified Client's requestRaw
        const response = await client.requestRaw(targetPath + req.nextUrl.search, {
            method: req.method,
            headers: forwardHeaders,
            body,
            cache: 'no-store'
        });

        // Read response body for error handling logging
        // Note: Reading body here consumes the stream if we don't clone.
        // UnifiedApiClient.requestRaw returns the raw fetch Response.
        // If we want to log errors from body, we need to clone or text().

        let responseBody: BodyInit | null = response.body;

        // Check for error status to log details (similar to original code)
        if (!response.ok) {
            // Clone to read text
            const clone = response.clone();
            const text = await clone.text();
            console.error(`❌ Backend Error ${response.status} ${req.method} ${targetPath}:`, {
                status: response.status,
                statusText: response.statusText,
                body: text.substring(0, 500),
                url: response.url
            });
            // We can still return response.body as it wasn't consumed (we used clone)
        }

        // Return response
        return new NextResponse(responseBody, {
            status: response.status,
            statusText: response.statusText,
            headers: response.headers
        });

    } catch (error) {
        console.error('❌ API Proxy Error:', error);
        return NextResponse.json(
            {
                error: 'Proxy implementation failed',
                message: error instanceof Error ? error.message : String(error),
                details: error instanceof Error ? error.stack : undefined
            },
            { status: 500 }
        );
    }
}
