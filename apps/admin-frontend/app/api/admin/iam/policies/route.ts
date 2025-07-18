import { NextResponse } from 'next/server';

export async function GET() {
  try {
    // For now, return mock data. In production, this would query the database
    const policies = [
      {
        id: '1',
        name: 'BronzePolicy',
        description: 'Basic access policy for Bronze tier users',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            {
              Effect: 'Allow',
              Action: ['dashboard:read', 'profile:read', 'profile:update'],
              Resource: ['user:self']
            }
          ]
        },
        arn: 'arn:epsx:iam::policy/BronzePolicy',
        path: '/',
        isAttachable: true,
        attachmentCount: 1,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '2',
        name: 'SilverPolicy',
        description: 'Premium access policy for Silver tier users',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            {
              Effect: 'Allow',
              Action: ['dashboard:read', 'profile:read', 'profile:update', 'analytics:read'],
              Resource: ['user:self', 'analytics:basic']
            }
          ]
        },
        arn: 'arn:epsx:iam::policy/SilverPolicy',
        path: '/',
        isAttachable: true,
        attachmentCount: 1,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '3',
        name: 'GoldPolicy',
        description: 'Advanced access policy for Gold tier users',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            {
              Effect: 'Allow',
              Action: ['dashboard:read', 'profile:read', 'profile:update', 'analytics:read', 'trading:read'],
              Resource: ['user:self', 'analytics:*', 'trading:*']
            }
          ]
        },
        arn: 'arn:epsx:iam::policy/GoldPolicy',
        path: '/',
        isAttachable: true,
        attachmentCount: 1,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '4',
        name: 'PlatinumPolicy',
        description: 'Premium access policy for Platinum tier users',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            {
              Effect: 'Allow',
              Action: ['dashboard:*', 'profile:*', 'analytics:*', 'trading:*', 'api:read'],
              Resource: ['user:self', 'analytics:*', 'trading:*', 'api:*']
            }
          ]
        },
        arn: 'arn:epsx:iam::policy/PlatinumPolicy',
        path: '/',
        isAttachable: true,
        attachmentCount: 1,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '5',
        name: 'AdminPolicy',
        description: 'Full administrative access policy',
        policyDocument: {
          Version: '2012-10-17',
          Statement: [
            {
              Effect: 'Allow',
              Action: ['*'],
              Resource: ['*']
            }
          ]
        },
        arn: 'arn:epsx:iam::policy/AdminPolicy',
        path: '/',
        isAttachable: true,
        attachmentCount: 1,
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    return NextResponse.json({ policies });
  } catch (error) {
    console.error('Failed to fetch policies:', error);
    return NextResponse.json(
      { error: 'Failed to fetch policies' },
      { status: 500 }
    );
  }
}

export async function POST(request: Request) {
  try {
    const body = await request.json();
    const { name, description, policyDocument } = body;

    // TODO: Implement actual policy creation in database
    // For now, return success
    const newPolicy = {
      id: Date.now().toString(),
      name,
      description,
      policyDocument,
      arn: `arn:epsx:iam::policy/${name}`,
      path: '/',
      isAttachable: true,
      attachmentCount: 0,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    return NextResponse.json({ policy: newPolicy });
  } catch (error) {
    console.error('Failed to create policy:', error);
    return NextResponse.json(
      { error: 'Failed to create policy' },
      { status: 500 }
    );
  }
}
