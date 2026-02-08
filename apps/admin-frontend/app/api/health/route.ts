/**
 * Health Check API Route
 * Used by Docker health check to verify the admin frontend is running
 */
import { NextResponse } from 'next/server';

/**
 *
 */
export function GET(): NextResponse {
    return NextResponse.json(
        {
            status: 'healthy',
            service: 'epsx-admin-frontend',
            timestamp: new Date().toISOString()
        },
        { status: 200 }
    );
}
