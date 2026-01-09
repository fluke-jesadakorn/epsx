/**
 * Health Check API Route
 * Used by Docker health check to verify the admin frontend is running
 */
import { NextResponse } from 'next/server';

/**
 *
 */
export async function GET(): Promise<NextResponse> {
    return NextResponse.json(
        {
            status: 'healthy',
            service: 'epsx-admin-frontend',
            timestamp: new Date().toISOString()
        },
        { status: 200 }
    );
}
