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

    // Extract extension data from request body
    const body = await request.json();
    
    if (!body.expiresAt) {
      return NextResponse.json(
        { error: 'New expiration date is required' },
        { status: 400 }
      );
    }

    // Validate the expiration date
    const newExpirationDate = new Date(body.expiresAt);
    if (isNaN(newExpirationDate.getTime())) {
      return NextResponse.json(
        { error: 'Invalid expiration date format' },
        { status: 400 }
      );
    }

    // Check if the new date is in the future
    if (newExpirationDate <= new Date()) {
      return NextResponse.json(
        { error: 'Expiration date must be in the future' },
        { status: 400 }
      );
    }

    // Prepare payload for backend
    const backendPayload = {
      expires_at: newExpirationDate.toISOString(),
      extended_by: body.extendedBy || 'current_admin', // This should come from auth context
      extension_reason: body.reason || 'Extended by admin'
    };

    // Forward request to backend API
    const backendUrl = process.env.BACKEND_API_URL || 'http://localhost:8080';
    const response = await fetch(`${backendUrl}/api/admin/stock-ranking/assignments/${assignmentId}/extend`, {
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
        { error: backendResult.error || 'Failed to extend assignment' },
        { status: response.status }
      );
    }

    return NextResponse.json({
      success: true,
      message: 'Assignment extended successfully',
      assignmentId,
      newExpirationDate: newExpirationDate.toISOString(),
      extendedAt: new Date().toISOString()
    });

  } catch (error) {
    console.error('Extend stock ranking assignment error:', error);
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