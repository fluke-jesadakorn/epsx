import { NextRequest, NextResponse } from 'next/server';
import { markAllNotificationsRead } from '@/lib/admin-server-data';

export async function POST(request: NextRequest) {
  try {
    // Mark all notifications as read for the current admin user
    const result = await markAllNotificationsRead();
    
    return NextResponse.json({
      success: true,
      message: 'All notifications marked as read',
      updated_count: result.updated_count || 0
    });
  } catch (error) {
    console.error('Error marking all notifications as read:', error);
    
    return NextResponse.json(
      { 
        error: 'Failed to mark all notifications as read',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}