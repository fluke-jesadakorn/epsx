import { NextRequest, NextResponse } from 'next/server';

// Mock data - replace with actual database operations
const mockUsers = [
  {
    id: '1',
    email: 'user1@example.com',
    name: 'John Doe',
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
      }
    ],
    customPermissions: [],
    effectivePermissions: [],
  },
  {
    id: '2',
    email: 'user2@example.com',
    name: 'Jane Smith',
    packageTier: 'silver',
    subscriptionStatus: 'active',
    lastPaymentDate: new Date('2024-01-10'),
    packagePermissions: [
      {
        id: 'silver_1',
        packageTier: 'silver',
        featureId: 'api_company',
        permission: { action: 'execute', resource: 'api:company' },
        isDefault: true,
        autoGranted: true,
      }
    ],
    customPermissions: [
      {
        id: 'custom_1',
        userId: '2',
        featureId: 'beta_features',
        permission: { action: 'view', resource: 'features:beta' },
        grantedBy: 'admin',
        grantedAt: new Date('2024-01-20'),
        reason: 'Beta tester program',
        isActive: true,
      }
    ],
    effectivePermissions: [],
  }
];

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const packageTier = searchParams.get('packageTier');
    const subscriptionStatus = searchParams.get('subscriptionStatus');
    const hasCustomPermissions = searchParams.get('hasCustomPermissions');

    let filteredUsers = [...mockUsers];

    if (packageTier) {
      filteredUsers = filteredUsers.filter(user => user.packageTier === packageTier);
    }

    if (subscriptionStatus) {
      filteredUsers = filteredUsers.filter(user => user.subscriptionStatus === subscriptionStatus);
    }

    if (hasCustomPermissions === 'true') {
      filteredUsers = filteredUsers.filter(user => user.customPermissions.length > 0);
    }

    return NextResponse.json(filteredUsers);
  } catch (error) {
    console.error('Error fetching users:', error);
    return NextResponse.json(
      { error: 'Failed to fetch users' },
      { status: 500 }
    );
  }
}
