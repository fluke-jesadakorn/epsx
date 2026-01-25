
import { logger } from '@/lib/logger';
import { COOKIES } from '@/shared/auth/cookies';
import { createAdminApiClient } from '@/shared/utils/api-client';
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

        logger.debug(`🔄 API Proxy: ${req.method} ${targetPath}`, {
            hasToken: !!token
        });

        // Initialize Unified Client (Server Side)
        // It handles Base URL resolution and Token extraction from cookies automatically
        const client = createAdminApiClient({ serverSide: true });

        // Create headers to forward
        const forwardHeaders = new Headers();

        // Copy safe headers
        const allowedHeaders = ['accept', 'content-type', 'user-agent', 'x-forwarded-for', 'x-forwarded-proto', 'origin', 'referer'];
        req.headers.forEach((value, key) => {
            if (allowedHeaders.includes(key.toLowerCase()) || key.startsWith('x-')) {
                forwardHeaders.set(key, value);
            }
        });

        // Remove cookie to prevent sending it twice (UnifiedClient handles extraction)
        forwardHeaders.delete('cookie');

        // Prepare body (for non-GET/HEAD requests)
        const body = (req.method !== 'GET' && req.method !== 'HEAD')
            ? await req.blob()  // Request body as Blob
            : undefined;

        // Use Unified Client's requestRaw to handle the proxying
        const response = await client.requestRaw(targetPath + req.nextUrl.search, {
            method: req.method,
            headers: forwardHeaders,
            body: body as any, // Cast blob to BodyInit
            cache: 'no-store'
        });

        // Stream response back
        return new NextResponse(response.body, {
            status: response.status,
            statusText: response.statusText,
            headers: response.headers
        });

    } catch (error) {
        logger.error('❌ API Proxy Error:', { error: String(error) });
        return NextResponse.json(
            { error: 'Proxy implementation failed', details: String(error) },
            { status: 500 }
        );
    }
}
