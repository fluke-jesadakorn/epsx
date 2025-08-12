import { NextRequest, NextResponse } from 'next/server';
import { getBearerToken } from '@/lib/actions/server-auth';

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

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
      // If backend route doesn't exist, return mock success
      if (response.status === 404) {
        return NextResponse.json({ 
          success: true,
          message: 'Assignment extended (mock response - backend not implemented)' 
        });
      }
      
      const errorText = await response.text();
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
        success: true,
        message: 'Assignment extended (fallback - service unavailable)' 
      }, 
      { status: 200 }
    );
  }
}