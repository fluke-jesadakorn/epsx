import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { userId, action, performedBy, reason, timestamp } = body;

    // Mock audit log creation
    const auditEntry = {
      id: `audit_${Date.now()}`,
      userId,
      action,
      performedBy,
      reason,
      timestamp,
    };

    console.log('Audit log entry:', auditEntry);

    return NextResponse.json({
      success: true,
      auditId: auditEntry.id,
    });
  } catch (error) {
    console.error('Error creating audit log:', error);
    return NextResponse.json(
      { error: 'Failed to create audit log' },
      { status: 500 }
    );
  }
}
