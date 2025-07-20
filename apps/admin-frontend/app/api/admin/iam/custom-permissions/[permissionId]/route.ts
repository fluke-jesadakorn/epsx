import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  _request: NextRequest,
  { params }: { params: { permissionId: string } }
) {
  try {
    const permissionId = params.permissionId;
    
    // Mock get custom permission operation
    const mockPermission = {
      id: permissionId,
      userId: 'test-user',
      featureId: 'beta_features',
      permission: { action: 'view', resource: 'features:beta' },
      grantedBy: 'admin',
      grantedAt: new Date('2024-01-20'),
      reason: 'Beta tester program',
      isActive: true,
    };

    return NextResponse.json(mockPermission);
  } catch (error) {
    console.error('Error fetching custom permission:', error);
    return NextResponse.json(
      { error: 'Failed to fetch custom permission' },
      { status: 500 }
    );
  }
}

export async function DELETE(
  request: NextRequest,
  { params }: { params: { permissionId: string } }
) {
  try {
    const permissionId = params.permissionId;
    const body = await request.json();
    const { revokedBy, reason } = body;

    // Mock revoke custom permission operation
    console.log(`Revoking permission ${permissionId} by ${revokedBy}. Reason: ${reason}`);

    return NextResponse.json({
      success: true,
      message: `Permission ${permissionId} revoked successfully`,
    });
  } catch (error) {
    console.error('Error revoking custom permission:', error);
    return NextResponse.json(
      { error: 'Failed to revoke custom permission' },
      { status: 500 }
    );
  }
}
