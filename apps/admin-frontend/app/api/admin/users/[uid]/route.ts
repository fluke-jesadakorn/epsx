import { NextRequest, NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';

interface AdminUser {
  uid: string;
  email: string;
  emailVerified: boolean;
  displayName?: string;
  disabled: boolean;
  customClaims?: any;
  metadata: {
    creationTime?: string;
    lastSignInTime?: string;
    lastRefreshTime?: string | null;
  };
}

export async function GET(
  _request: NextRequest,
  { params }: { params: { uid: string } }
) {
  try {
    const auth = getAuthAdmin();
    const { uid } = params;

    const userRecord = await auth.getUser(uid);
    
    const user: AdminUser = {
      uid: userRecord.uid,
      email: userRecord.email || '',
      emailVerified: userRecord.emailVerified,
      displayName: userRecord.displayName,
      disabled: userRecord.disabled,
      customClaims: userRecord.customClaims,
      metadata: {
        creationTime: userRecord.metadata.creationTime,
        lastSignInTime: userRecord.metadata.lastSignInTime,
        lastRefreshTime: userRecord.metadata.lastRefreshTime,
      },
    };

    return NextResponse.json(user);
  } catch (error: any) {
    console.error(`Failed to get user ${params.uid}:`, error);
    return NextResponse.json(
      { error: error.message || 'Failed to fetch user' },
      { status: 500 }
    );
  }
}
