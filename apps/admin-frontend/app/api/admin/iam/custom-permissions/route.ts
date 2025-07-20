import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const customPermission = body;

    // Mock create custom permission operation
    const newPermission = {
      id: `custom_${Date.now()}`,
      ...customPermission,
    };

    console.log('Creating custom permission:', newPermission);

    return NextResponse.json(newPermission);
  } catch (error) {
    console.error('Error creating custom permission:', error);
    return NextResponse.json(
      { error: 'Failed to create custom permission' },
      { status: 500 }
    );
  }
}

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const userId = searchParams.get('userId');

    // Mock get custom permissions operation
    const mockPermissions = [
      {
        id: 'custom_1',
        userId: userId || 'test',
        featureId: 'beta_features',
        permission: { action: 'view', resource: 'features:beta' },
        grantedBy: 'admin',
        grantedAt: new Date('2024-01-20'),
        reason: 'Beta tester program',
        isActive: true,
      }
    ];

    return NextResponse.json(mockPermissions);
  } catch (error) {
    console.error('Error fetching custom permissions:', error);
    return NextResponse.json(
      { error: 'Failed to fetch custom permissions' },
      { status: 500 }
    );
  }
}
