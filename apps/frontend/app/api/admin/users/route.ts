import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';
import { getUserCustomClaims } from '@/lib/custom-claims';
import { verifySession } from '@/lib/session';
import { UserRole } from '@/types/auth/roles';

export async function GET(request: NextRequest) {
  try {
    // Verify the requesting user is authenticated and has admin role
    const session = await verifySession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // Get requesting user's claims to verify admin access
    const userClaims = await getUserCustomClaims(session.uid);
    if (!userClaims || userClaims.role !== UserRole.ADMIN) {
      return NextResponse.json({ error: 'Admin access required' }, { status: 403 });
    }

    const auth = getAuthAdmin();
    const { searchParams } = new URL(request.url);
    const limit = Math.min(parseInt(searchParams.get('limit') || '50'), 100);
    const pageToken = searchParams.get('pageToken') || undefined;

    // List users from Firebase Auth
    const listUsersResult = await auth.listUsers(limit, pageToken);
    
    // Enrich user data with custom claims
    const enrichedUsers = await Promise.all(
      listUsersResult.users.map(async (user) => {
        try {
          const claims = await getUserCustomClaims(user.uid);
          return {
            uid: user.uid,
            email: user.email || '',
            emailVerified: user.emailVerified,
            displayName: user.displayName,
            photoURL: user.photoURL,
            disabled: user.disabled,
            claims,
            lastSignIn: user.metadata.lastSignInTime,
            createdAt: user.metadata.creationTime,
            provider: user.providerData.map(p => p.providerId),
          };
        } catch (error) {
          console.error(`Failed to get claims for user ${user.uid}:`, error);
          return {
            uid: user.uid,
            email: user.email || '',
            emailVerified: user.emailVerified,
            displayName: user.displayName,
            photoURL: user.photoURL,
            disabled: user.disabled,
            claims: null,
            lastSignIn: user.metadata.lastSignInTime,
            createdAt: user.metadata.creationTime,
            provider: user.providerData.map(p => p.providerId),
          };
        }
      })
    );

    return NextResponse.json({
      users: enrichedUsers,
      nextPageToken: listUsersResult.pageToken,
      totalUsers: enrichedUsers.length,
    });
  } catch (error) {
    console.error('Error fetching users:', error);
    return NextResponse.json(
      { error: 'Failed to fetch users' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    // Verify the requesting user is authenticated and has admin role
    const session = await verifySession();
    if (!session) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // Get requesting user's claims to verify admin access
    const userClaims = await getUserCustomClaims(session.uid);
    if (!userClaims || userClaims.role !== UserRole.ADMIN) {
      return NextResponse.json({ error: 'Admin access required' }, { status: 403 });
    }

    const body = await request.json();
    const { uid, action, ...params } = body;

    if (!uid || !action) {
      return NextResponse.json(
        { error: 'Missing required fields: uid, action' },
        { status: 400 }
      );
    }

    const auth = getAuthAdmin();

    switch (action) {
      case 'disable':
        await auth.updateUser(uid, { disabled: true });
        return NextResponse.json({ success: true, message: 'User disabled' });

      case 'enable':
        await auth.updateUser(uid, { disabled: false });
        return NextResponse.json({ success: true, message: 'User enabled' });

      case 'delete':
        await auth.deleteUser(uid);
        return NextResponse.json({ success: true, message: 'User deleted' });

      case 'reset_password':
        if (!params.email) {
          return NextResponse.json(
            { error: 'Email required for password reset' },
            { status: 400 }
          );
        }
        // Generate password reset link (this requires additional setup)
        const resetLink = await auth.generatePasswordResetLink(params.email);
        return NextResponse.json({ 
          success: true, 
          message: 'Password reset link generated',
          resetLink 
        });

      default:
        return NextResponse.json(
          { error: 'Invalid action' },
          { status: 400 }
        );
    }
  } catch (error) {
    console.error('Error performing user action:', error);
    return NextResponse.json(
      { error: 'Failed to perform action' },
      { status: 500 }
    );
  }
}
