import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from '@/lib/auth/server-auth';
import { env } from '@/config/env';

// Get bearer token from custom JWT session
const getBearerToken = async () => {
  const session = await getServerSession();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = env.NEXT_PUBLIC_BACKEND_URL;

interface RouteParams {
  params: {
    assignmentId: string;
  };
}

export async function POST(request: NextRequest, { params }: RouteParams) {
  try {
    const token = await getBearerToken();
    const { assignmentId } = await params;
    const body = await request.json().catch(() => ({}));
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/stock-ranking/assignments/${assignmentId}/extend`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Backend stock ranking extend API failed: ${response.status} ${errorText}`);
      return NextResponse.json(
        { error: `Backend error: ${errorText}` }, 
        { status: response.status }
      );
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('Stock ranking extend API error:', error);
    return NextResponse.json(
      { 
        success: false,
        error: 'Failed to extend assignment',
        message: 'Service temporarily unavailable' 
      }, 
      { status: 500 }
    );
  }
}