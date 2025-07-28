import { NextResponse } from 'next/server';

export async function GET() {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      version: process.env.npm_package_version || '1.0.0',
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      environment: process.env.NODE_ENV,
      checks: {
        server: true,
        database: await checkDatabase(),
        cache: await checkCache(),
        external_apis: await checkExternalAPIs(),
      }
    };

    const allHealthy = Object.values(health.checks).every(Boolean);
    
    return NextResponse.json(
      { ...health, status: allHealthy ? 'healthy' : 'degraded' },
      { status: allHealthy ? 200 : 503 }
    );
  } catch (error) {
    console.error('Health check failed:', error);
    return NextResponse.json(
      {
        status: 'unhealthy',
        timestamp: new Date().toISOString(),
        error: error instanceof Error ? error.message : 'Unknown error',
      },
      { status: 500 }
    );
  }
}

async function checkDatabase(): Promise<boolean> {
  try {
    // Add database connectivity check here
    // For now, assume healthy
    return true;
  } catch {
    return false;
  }
}

async function checkCache(): Promise<boolean> {
  try {
    // Add cache connectivity check here
    // For now, assume healthy
    return true;
  } catch {
    return false;
  }
}

async function checkExternalAPIs(): Promise<boolean> {
  try {
    // Add external API health checks here
    // For now, assume healthy
    return true;
  } catch {
    return false;
  }
}