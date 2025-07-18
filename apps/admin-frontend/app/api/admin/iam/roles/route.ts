import { NextResponse } from 'next/server';

export async function GET() {
  try {
    // For now, return mock data. In production, this would query the database
    const roles = [
      {
        id: '1',
        name: 'Bronze',
        description: 'Basic tier access with limited permissions',
        attachedPolicies: ['BronzePolicy'],
        path: '/',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '2',
        name: 'Silver',
        description: 'Premium tier access with enhanced permissions',
        attachedPolicies: ['SilverPolicy'],
        path: '/',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '3',
        name: 'Gold',
        description: 'Advanced tier access with extended permissions',
        attachedPolicies: ['GoldPolicy'],
        path: '/',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '4',
        name: 'Platinum',
        description: 'Premium tier with comprehensive permissions',
        attachedPolicies: ['PlatinumPolicy'],
        path: '/',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
      {
        id: '5',
        name: 'Admin',
        description: 'Full administrative access to all resources',
        attachedPolicies: ['AdminPolicy'],
        path: '/',
        createdAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      },
    ];

    return NextResponse.json({ roles });
  } catch (error) {
    console.error('Failed to fetch roles:', error);
    return NextResponse.json(
      { error: 'Failed to fetch roles' },
      { status: 500 }
    );
  }
}

export async function POST(request: Request) {
  try {
    const body = await request.json();
    const { name, description, attachedPolicies } = body;

    // TODO: Implement actual role creation in database
    // For now, return success
    const newRole = {
      id: Date.now().toString(),
      name,
      description,
      attachedPolicies: attachedPolicies || [],
      path: '/',
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };

    return NextResponse.json({ role: newRole });
  } catch (error) {
    console.error('Failed to create role:', error);
    return NextResponse.json(
      { error: 'Failed to create role' },
      { status: 500 }
    );
  }
}
