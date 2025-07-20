import { NextRequest, NextResponse } from 'next/server';

export async function GET(
  _request: NextRequest,
  { params }: { params: { userId: string } }
) {
  try {
    const userId = params.userId;
    
    // Mock user data - replace with actual database query
    const mockUser = {
      id: userId,
      email: `user${userId}@example.com`,
      name: `User ${userId}`,
      packageTier: 'gold',
      subscriptionStatus: 'active',
      lastPaymentDate: new Date('2024-01-15'),
      packagePermissions: [
        {
          id: 'gold_1',
          packageTier: 'gold',
          featureId: 'api_partner',
          permission: { action: 'execute', resource: 'api:partner' },
          isDefault: true,
          autoGranted: true,
        },
        {
          id: 'gold_2',
          packageTier: 'gold',
          featureId: 'data_export',
          permission: { action: 'execute', resource: 'data:export' },
          isDefault: true,
          autoGranted: true,
        }
      ],
      customPermissions: [],
      effectivePermissions: [],
    };

    return NextResponse.json(mockUser);
  } catch (error) {
    console.error('Error fetching user:', error);
    return NextResponse.json(
      { error: 'Failed to fetch user' },
      { status: 500 }
    );
  }
}

export async function PATCH(
  request: NextRequest,
  { params }: { params: { userId: string } }
) {
  try {
    const userId = params.userId;
    const body = await request.json();
    const { packageTier, updatedBy } = body;

    // Mock update operation - replace with actual database update
    console.log(`Updating user ${userId} package tier to ${packageTier} by ${updatedBy}`);

    return NextResponse.json({
      success: true,
      message: `User ${userId} package tier updated to ${packageTier}`,
    });
  } catch (error) {
    console.error('Error updating user:', error);
    return NextResponse.json(
      { error: 'Failed to update user' },
      { status: 500 }
    );
  }
}
