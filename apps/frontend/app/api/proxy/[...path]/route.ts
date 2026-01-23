
import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

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

        // Construct backend URL
        // path is an array like ['api', 'admin', 'wallets', 'stats']
        const targetPath = '/' + path.join('/');
        const targetUrl = `${BACKEND_URL}${targetPath}${req.nextUrl.search}`;

        console.log(`🔄 API Proxy (Frontend): ${req.method} ${targetPath}`, {
            targetUrl,
            hasToken: !!token
        });

        // Create new headers object to control exactly what is sent
        const forwardHeaders = new Headers();

        // Copy safe headers
        const allowedHeaders = ['accept', 'content-type', 'user-agent', 'x-forwarded-for', 'x-forwarded-proto', 'origin', 'referer'];
        req.headers.forEach((value, key) => {
            if (allowedHeaders.includes(key.toLowerCase()) || key.startsWith('x-')) {
                forwardHeaders.set(key, value);
            }
        });

        // Ensure we don't send cookies to backend (it should only rely on Bearer token)
        forwardHeaders.delete('cookie');

        // Inject Authorization
        if (token) {
            forwardHeaders.set('Authorization', `Bearer ${token}`);
        } else {
            console.warn('⚠️ API Proxy: No token found for restricted endpoint');
        }

        // Prepare body (for non-GET/HEAD requests)
        // Prepare body (for non-GET/HEAD requests)
        let body = (req.method !== 'GET' && req.method !== 'HEAD')
            ? await req.blob()
            : undefined;

        // Special handling for token refresh: inject refresh token from HttpOnly cookie
        if (targetPath === '/api/auth/session/refresh' && req.method === 'POST') {
            const refreshToken = cookieStore.get(COOKIES.refresh_token)?.value;
            if (refreshToken) {
                console.log('🔄 API Proxy: Injecting refresh token from cookie');
                body = new Blob([JSON.stringify({ refresh_token: refreshToken })], { type: 'application/json' });
                forwardHeaders.set('content-type', 'application/json');
                // Ensure content-length is updated or handled by fetch automatically (blob handles it)
            } else {
                console.warn('⚠️ API Proxy: No refresh token cookie found for refresh request');
            }
        }

        // Forward request to backend
        const response = await fetch(targetUrl, {
            method: req.method,
            headers: forwardHeaders,
            body,
            cache: 'no-store'
        });

        // Read response body for error handling
        const responseBody = await response.text();
        
        // If there's an error, log it for debugging
        if (!response.ok) {
            console.error(`❌ Backend Error ${response.status} ${req.method} ${targetPath}:`, {
                status: response.status,
                statusText: response.statusText,
                body: responseBody.substring(0, 500), // Limit log size
                url: targetUrl
            });
        }

        // Return response with proper error body
        return new NextResponse(responseBody, {
            status: response.status,
            statusText: response.statusText,
            headers: {
                'Content-Type': response.headers.get('Content-Type') || 'application/json',
            }
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
