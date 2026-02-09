/**
 * Health Check API Route
 * Used by Docker health check to verify the frontend is running
 */
import { NextResponse } from 'next/server';

export function GET() {
    return NextResponse.json(
        {
            status: 'healthy',
            service: 'epsx-frontend',
            timestamp: new Date().toISOString()
        },
        { status: 200 }
    );
}
