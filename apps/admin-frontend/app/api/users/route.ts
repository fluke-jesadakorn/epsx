import { NextRequest, NextResponse } from 'next/server';
import { withAdminAccess } from '@/middleware/adminAccess';

// Example: User management API
async function handler(req: NextRequest) {
  try {
    const method = req.method;

    switch (method) {
      case 'GET':
        // Get all users
        const users = [
          { id: 1, email: 'user1@example.com', tier: 'GOLD' },
          { id: 2, email: 'user2@example.com', tier: 'SILVER' },
        ];
        return NextResponse.json({ success: true, data: users });

      case 'POST':
        // Create user (admin only)
        return NextResponse.json({ 
          success: true, 
          message: 'User created successfully' 
        });

      case 'DELETE':
        // Delete user (admin only)
        return NextResponse.json({ 
          success: true, 
          message: 'User deleted successfully' 
        });

      default:
        return NextResponse.json(
          { error: 'Method not allowed' },
          { status: 405 }
        );
    }
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to process request' },
      { status: 500 }
    );
  }
}

// Apply role-based middleware for different HTTP methods
export async function GET(req: NextRequest) {
  // Admins can read users
  const adminCheck = await withAdminAccess('users', 'read')(req);
  if (!adminCheck.ok) return adminCheck;
  
  return handler(req);
}

export async function POST(req: NextRequest) {
  // Admins can create users
  const adminCheck = await withAdminAccess('users', 'write')(req);
  if (!adminCheck.ok) return adminCheck;
  
  return handler(req);
}

export async function DELETE(req: NextRequest) {
  // Only admins can delete users
  const adminCheck = await withAdminAccess('users', 'delete')(req);
  if (!adminCheck.ok) return adminCheck;
  
  return handler(req);
}
