import { NextRequest, NextResponse } from 'next/server';
import { clearAllNotifications } from '@/lib/admin-server-data';

export async function DELETE(request: NextRequest) {
  try {
    // Clear all notifications for the current admin user
    const result = await clearAllNotifications();
    
    return NextResponse.json({
      success: true,
      message: 'All notifications cleared successfully',
      deleted_count: result.deleted_count || 0
    });
  } catch (error) {
    console.error('Error clearing all notifications:', error);
    
    return NextResponse.json(
      { 
        error: 'Failed to clear all notifications',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}