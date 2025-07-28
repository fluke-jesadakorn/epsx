import { NextRequest, NextResponse } from 'next/server';
import { headers } from 'next/headers';

export async function POST(request: NextRequest) {
  try {
    const performanceData = await request.json();
    const headersList = headers();
    
    // Log performance metrics (in production, send to monitoring service)
    console.log('Performance Metric:', {
      timestamp: new Date().toISOString(),
      userAgent: headersList.get('user-agent'),
      ...performanceData,
    });

    // In production, you would:
    // 1. Send to monitoring service (DataDog, New Relic, etc.)
    // 2. Store in database for analysis
    // 3. Alert on performance degradation
    
    // Example: Send to external monitoring service
    if (process.env.MONITORING_ENDPOINT) {
      await fetch(process.env.MONITORING_ENDPOINT, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${process.env.MONITORING_API_KEY}`,
        },
        body: JSON.stringify({
          service: 'epsx-frontend',
          type: 'performance',
          data: performanceData,
        }),
      });
    }

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Failed to process performance metric:', error);
    return NextResponse.json(
      { error: 'Failed to process metric' },
      { status: 500 }
    );
  }
}