import { NextResponse } from 'next/server';
import { getAuthAdmin } from '@/lib/firebase-admin';

interface UserStats {
  totalUsers: number;
  verifiedUsers: number;
  disabledUsers: number;
  adminUsers: number;
  verificationRate: number;
}

export async function GET() {
  try {
    const auth = getAuthAdmin();
    
    // Get users with a reasonable limit for stats calculation
    const result = await auth.listUsers(1000);
    const users = result.users;
    
    const totalUsers = users.length;
    const verifiedUsers = users.filter(user => user.emailVerified).length;
    const disabledUsers = users.filter(user => user.disabled).length;
    const adminUsers = users.filter(user => user.customClaims?.role === 'ADMIN').length;

    const stats: UserStats = {
      totalUsers,
      verifiedUsers,
      disabledUsers,
      adminUsers,
      verificationRate: totalUsers > 0 ? Math.round((verifiedUsers / totalUsers) * 100) : 0,
    };

    return NextResponse.json(stats);
  } catch (error: any) {
    console.error('Failed to get user stats:', error);
    return NextResponse.json(
      { error: error.message || 'Failed to fetch user statistics' },
      { status: 500 }
    );
  }
}
