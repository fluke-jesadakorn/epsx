import { NextRequest, NextResponse } from 'next/server';

export interface NotificationPreferences {
  trading: boolean;
  security: boolean;
  account: boolean;
  system: boolean;
  marketing: boolean;
}

// Mock storage for preferences (in real app, would use database)
const userPreferences = new Map<string, NotificationPreferences>();

export async function GET(request: NextRequest) {
  try {
    // In real implementation, get user ID from authenticated session
    const userId = 'current-user'; // Placeholder
    
    const preferences = userPreferences.get(userId) || {
      trading: true,
      security: true,
      account: true,
      system: false,
      marketing: false
    };

    return NextResponse.json({
      success: true,
      preferences
    });
  } catch (error) {
    console.error('Error getting notification preferences:', error);
    
    return NextResponse.json(
      { 
        error: 'Failed to get notification preferences',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const preferences: NotificationPreferences = await request.json();
    
    // Validate preferences
    const validCategories = ['trading', 'security', 'account', 'system', 'marketing'];
    for (const category of validCategories) {
      if (typeof preferences[category as keyof NotificationPreferences] !== 'boolean') {
        return NextResponse.json(
          { error: `Invalid value for ${category}. Must be boolean.` },
          { status: 400 }
        );
      }
    }

    // In real implementation, get user ID from authenticated session
    const userId = 'current-user'; // Placeholder
    
    // Save preferences (in real app, would save to database)
    userPreferences.set(userId, preferences);
    
    // TODO: Update FCM subscription with new category preferences
    // This would involve updating the FCM token registration with category filters

    return NextResponse.json({
      success: true,
      message: 'Notification preferences updated successfully',
      preferences
    });
  } catch (error) {
    console.error('Error updating notification preferences:', error);
    
    return NextResponse.json(
      { 
        error: 'Failed to update notification preferences',
        details: error instanceof Error ? error.message : 'Unknown error'
      },
      { status: 500 }
    );
  }
}