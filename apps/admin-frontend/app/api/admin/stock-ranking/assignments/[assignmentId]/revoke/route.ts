import { NextRequest, NextResponse } from 'next/server';

interface RouteParams {
  params: {
    assignmentId: string;
  };
}

export async function POST(request: NextRequest, { params }: RouteParams) {
  try {
    const { assignmentId } = params;

    if (!assignmentId) {
      return NextResponse.json(
        { error: 'Assignment ID is required' },
        { status: 400 }
      );
    }

    // Extract any additional data from request body
    const body = await request.json().catch(() => ({}));
    const reason = body.reason || 'Revoked by admin';

    // Prepare payload for backend
    const backendPayload = {
      reason,
      revoked_by: body.revokedBy || 'current_admin' // This should come from auth context
    };

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assignments/${assignmentId}/revoke`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': request.headers.get('Authorization') || '',
      },
      body: JSON.stringify(backendPayload),
    });

    const backendResult = await response.json();

    if (!response.ok) {
      return NextResponse.json(
        { error: backendResult.error || 'Failed to revoke assignment' },
        { status: response.status }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Assignment revoked successfully',
      assignmentId,
      revokedAt: new Date().toISOString()
    });

  } catch (error) {
    console.error('Revoke stock ranking assignment error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  return NextResponse.json(
    { error: 'Method not allowed' },
    { status: 405 }
  );
}