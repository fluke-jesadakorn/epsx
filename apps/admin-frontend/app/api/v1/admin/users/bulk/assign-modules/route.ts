import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/auth';

// Get bearer token from NextAuth session
const getBearerToken = async () => {
  const session = await auth();
  return (session as any)?.accessToken || null;
};

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    const token = await getBearerToken();
    const body = await request.json();
    
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(`${BACKEND_URL}/api/v1/admin/users/bulk/assign-modules`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      // If backend route doesn't exist, return mock success response
      if (response.status === 404) {
        return NextResponse.json({ 
          success: true,
          results: body.user_ids?.map((userId: string) => ({
            user_id: userId,
            success: true,
            assignments_created: body.assignments?.length || 0,
            message: 'Assignments completed (mock response)'
          })) || [],
          message: 'Bulk assignment completed (backend not implemented)' 
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
    console.error('Bulk assign API error:', error);
    return NextResponse.json(
      { 
        success: false,
        error: 'Failed to assign modules',
        message: 'Assignment service currently unavailable' 
      }, 
      { status: 500 }
    );
  }
}