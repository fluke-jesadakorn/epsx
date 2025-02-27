import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';
import { auth } from '@/lib/firebase-admin';

export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('session')?.value;

    if (!sessionCookie) {
      return NextResponse.json({ roles: [] }, { status: 401 });
    }

    const decodedClaim = await auth().verifySessionCookie(sessionCookie, true);

    if (!decodedClaim) {
      return NextResponse.json({ roles: [] }, { status: 401 });
    }

    const user = await auth().getUser(decodedClaim.uid);
    const roles = user.customClaims?.roles || [];

    return NextResponse.json({ roles });
  } catch (error) {
    console.error('Role verification error:', error);
    return NextResponse.json({ roles: [] }, { status: 401 });
  }
}

export async function POST(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('session')?.value;

    if (!sessionCookie) {
      return NextResponse.json({ success: false }, { status: 401 });
    }

    const decodedClaim = await auth().verifySessionCookie(sessionCookie, true);

    if (!decodedClaim || !decodedClaim.admin) {
      return NextResponse.json(
        { error: 'Unauthorized' },
        { status: 403 }
      );
    }

    const { userId, roles } = await request.json();

    if (!userId || !Array.isArray(roles)) {
      return NextResponse.json(
        { error: 'Invalid request body' },
        { status: 400 }
      );
    }

    await auth().setCustomUserClaims(userId, { roles });

    return NextResponse.json({ success: true });
  } catch (error) {
    console.error('Role update error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
